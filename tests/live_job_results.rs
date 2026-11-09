//! Job results against a live cluster: user-task listener corrections (including
//! clearing dates) and denial, and an ad-hoc sub-process result, each driven through the
//! public job worker API.
//!
//! These need a running, unauthenticated Camunda 8.8+ cluster, so they are `#[ignore]`d
//! and run explicitly:
//!
//! ```text
//! make test-live
//! # or: cargo test --test live_job_results -- --ignored
//! ```
//!
//! The client is built from the environment like any other (`CAMUNDA_REST_ADDRESS`,
//! default `http://localhost:8080`; `CAMUNDA_AUTH_STRATEGY`, default `NONE`). Every
//! process id and job type carries a random suffix, so the tests are safe to run
//! concurrently with each other and with other clients of the same cluster. Reads go
//! through the SDK's eventual-consistency polling; nothing sleeps.

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use camunda_orchestration_sdk::apis::element_instance_api::SearchElementInstancesParams;
use camunda_orchestration_sdk::apis::user_task_api::{
    CompleteUserTaskParams, SearchUserTasksParams,
};
use camunda_orchestration_sdk::eventual::ConsistencyOptions;
use camunda_orchestration_sdk::models::{
    self, ElementInstanceFilter, ElementInstanceSearchQuery, ProcessDefinitionId,
    ProcessInstanceCreationInstruction, ProcessInstanceCreationInstructionById, ProcessInstanceKey,
    ProcessInstanceKeyExactMatch, ProcessInstanceKeyFilterProperty, SearchQueryPageRequest,
    UserTaskCompletionRequest, UserTaskFilter, UserTaskSearchQuery,
};
use camunda_orchestration_sdk::{
    AdHocSubProcessResult, CamundaClient, CamundaError, JobAction, JobWorkerConfig,
    JobWorkerHandle, LiveRandom, Random, UserTaskCorrections, UserTaskResult,
};

/// The BPMN due / follow-up dates the corrections scenario clears.
const DUE_DATE: &str = "2030-01-01T10:00:00Z";
const FOLLOW_UP_DATE: &str = "2029-12-01T10:00:00Z";

fn client() -> CamundaClient {
    CamundaClient::from_env().expect("client builds from the environment")
}

/// A unique, BPMN-id-safe name: `rust-sdk-<what>-<random hex>`.
fn unique(what: &str) -> String {
    let suffix = (LiveRandom.next_f64() * (1u64 << 53) as f64) as u64;
    format!("rust-sdk-{what}-{suffix:014x}")
}

fn poll() -> ConsistencyOptions {
    ConsistencyOptions::default().timeout(Duration::from_secs(30))
}

async fn deploy(client: &CamundaClient, process_id: &str, bpmn: String) {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let path = dir.join(format!("{process_id}.bpmn"));
    std::fs::write(&path, bpmn).expect("write the BPMN");
    let deployed = client.deploy_resources(vec![path.clone()], None).await;
    let _ = std::fs::remove_file(&path);
    deployed.expect("deploy the process");
}

async fn start(client: &CamundaClient, process_id: &str) -> String {
    let instruction = ProcessInstanceCreationInstruction::ProcessInstanceCreationInstructionById(
        Box::new(ProcessInstanceCreationInstructionById {
            process_definition_id: ProcessDefinitionId::assume_exists(process_id),
            ..Default::default()
        }),
    );
    let created = client
        .create_process_instance(instruction)
        .await
        .expect("start a process instance");
    created.process_instance_key.value().to_string()
}

fn worker(
    client: &CamundaClient,
    job_type: &str,
    handler: impl Fn(models::ActivatedJobResult) -> JobAction + Send + Sync + 'static,
) -> JobWorkerHandle {
    let handler = Arc::new(handler);
    client
        .create_job_worker(JobWorkerConfig::new(job_type).worker_name(unique("worker")))
        .spawn(move |job| {
            let handler = handler.clone();
            async move { handler(job.raw().clone()) }
        })
}

/// The user tasks of one process instance, once `ready` holds for them.
async fn user_tasks_until(
    client: &CamundaClient,
    process_instance_key: &str,
    ready: impl Fn(&[models::UserTaskResult]) -> bool,
) -> Vec<models::UserTaskResult> {
    let query = UserTaskSearchQuery {
        filter: Some(Box::new(UserTaskFilter {
            process_instance_key: Some(Box::new(
                ProcessInstanceKeyFilterProperty::ProcessInstanceKeyExactMatch(Box::new(
                    ProcessInstanceKeyExactMatch::assume_exists(process_instance_key),
                )),
            )),
            ..Default::default()
        })),
        ..Default::default()
    };
    client
        .eventual_until(
            poll(),
            || {
                let query = query.clone();
                async move {
                    client
                        .search_user_tasks(SearchUserTasksParams {
                            user_task_search_query: Some(query),
                        })
                        .await
                }
            },
            |result| ready(&result.items),
        )
        .await
        .expect("user tasks become visible")
        .items
}

/// The element instances of one process instance, once `ready` holds for them.
async fn element_instances_until(
    client: &CamundaClient,
    process_instance_key: &str,
    ready: impl Fn(&[models::ElementInstanceResult]) -> bool,
) -> Vec<models::ElementInstanceResult> {
    let query = ElementInstanceSearchQuery {
        filter: Some(Box::new(ElementInstanceFilter {
            process_instance_key: Some(Box::new(ProcessInstanceKey::assume_exists(
                process_instance_key,
            ))),
            ..Default::default()
        })),
        page: Some(Box::new(SearchQueryPageRequest::LimitPagination(Box::new(
            models::LimitPagination { limit: Some(100) },
        )))),
        ..Default::default()
    };
    client
        .eventual_until(
            poll(),
            || {
                let query = query.clone();
                async move {
                    client
                        .search_element_instances(SearchElementInstancesParams {
                            element_instance_search_query: Some(query),
                        })
                        .await
                }
            },
            |result| ready(&result.items),
        )
        .await
        .expect("element instances become visible")
        .items
}

/// A process with one Camunda user task carrying a due / follow-up date and a task
/// listener of `listener_job_type` on `event_type`.
fn user_task_process(process_id: &str, event_type: &str, listener_job_type: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:zeebe="http://camunda.org/schema/zeebe/1.0"
                  id="defs-{process_id}" targetNamespace="http://bpmn.io/schema/bpmn">
  <bpmn:process id="{process_id}" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:sequenceFlow id="to-task" sourceRef="start" targetRef="review" />
    <bpmn:userTask id="review" name="Review">
      <bpmn:extensionElements>
        <zeebe:userTask />
        <zeebe:taskSchedule dueDate="{DUE_DATE}" followUpDate="{FOLLOW_UP_DATE}" />
        <zeebe:taskListeners>
          <zeebe:taskListener eventType="{event_type}" type="{listener_job_type}" />
        </zeebe:taskListeners>
      </bpmn:extensionElements>
    </bpmn:userTask>
    <bpmn:sequenceFlow id="to-end" sourceRef="review" targetRef="end" />
    <bpmn:endEvent id="end" />
  </bpmn:process>
</bpmn:definitions>
"#
    )
}

#[tokio::test]
#[ignore = "needs a live Camunda cluster; run with `make test-live`"]
async fn creating_listener_corrects_the_task_and_clears_its_dates() {
    let client = client();
    let process_id = unique("corrections");
    let job_type = unique("creating-listener");
    deploy(
        &client,
        &process_id,
        user_task_process(&process_id, "creating", &job_type),
    )
    .await;

    // The listener job reports the task as the BPMN defined it, so the test can show the
    // dates were there before the corrections cleared them.
    let (seen_tx, mut seen_rx) = tokio::sync::mpsc::unbounded_channel();
    let handle = worker(&client, &job_type, move |job| {
        let _ = seen_tx.send(job.user_task.clone());
        JobAction::complete_with_result(
            UserTaskResult::new().corrections(
                UserTaskCorrections::new()
                    .assignee("rust-sdk-corrected-user")
                    .priority(83)
                    .clear_due_date()
                    .clear_follow_up_date(),
            ),
        )
    });

    let key = start(&client, &process_id).await;
    let tasks = user_tasks_until(&client, &key, |tasks| {
        tasks
            .iter()
            .any(|t| t.state == models::UserTaskStateEnum::Created)
    })
    .await;
    handle.stop();

    let before = seen_rx
        .recv()
        .await
        .flatten()
        .expect("a creating listener job carries the user task");
    assert!(
        before.due_date.is_some() && before.follow_up_date.is_some(),
        "the BPMN dates are set before the corrections: {before:?}"
    );

    let [task] = tasks.as_slice() else {
        panic!("expected one user task, got {tasks:?}");
    };
    assert_eq!(task.assignee.as_deref(), Some("rust-sdk-corrected-user"));
    assert_eq!(task.priority, 83);
    assert_eq!(task.due_date, None, "dueDate is cleared");
    assert_eq!(task.follow_up_date, None, "followUpDate is cleared");
}

#[tokio::test]
#[ignore = "needs a live Camunda cluster; run with `make test-live`"]
async fn completing_listener_denial_keeps_the_task_open() {
    let client = client();
    let process_id = unique("denial");
    let job_type = unique("completing-listener");
    deploy(
        &client,
        &process_id,
        user_task_process(&process_id, "completing", &job_type),
    )
    .await;

    let denials = Arc::new(AtomicUsize::new(0));
    let counted = denials.clone();
    let handle = worker(&client, &job_type, move |_job| {
        counted.fetch_add(1, Ordering::SeqCst);
        JobAction::complete_with_result(UserTaskResult::deny("rust-sdk: missing documents"))
    });

    let key = start(&client, &process_id).await;
    let tasks = user_tasks_until(&client, &key, |tasks| {
        tasks
            .iter()
            .any(|t| t.state == models::UserTaskStateEnum::Created)
    })
    .await;
    let task_key = tasks[0].user_task_key.value().to_string();

    // The complete command waits for the completing listener; the denial rejects it.
    let completed = client
        .complete_user_task(CompleteUserTaskParams {
            user_task_key: task_key.clone(),
            user_task_completion_request: Some(UserTaskCompletionRequest::default()),
        })
        .await;
    handle.stop();

    match completed {
        Err(CamundaError::Api { status, body }) => {
            let body = body.unwrap_or_default();
            assert_eq!(status, 409, "a denied completion is a conflict: {body}");
            assert!(
                body.contains("denied by Task Listener")
                    && body.contains("rust-sdk: missing documents"),
                "the rejection carries the denial and its reason: {body}"
            );
        }
        other => panic!("expected the completion to be rejected, got {other:?}"),
    }
    assert_eq!(denials.load(Ordering::SeqCst), 1, "the listener ran once");

    // The task reverts to CREATED and stays incomplete.
    let tasks = user_tasks_until(&client, &key, |tasks| {
        tasks
            .iter()
            .any(|t| t.state == models::UserTaskStateEnum::Created)
    })
    .await;
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].completion_date, None);
    let instance = client
        .get_process_instance(&key)
        .await
        .expect("read the process instance");
    assert_eq!(instance.state, models::ProcessInstanceStateEnum::Active);
}

#[tokio::test]
#[ignore = "needs a live Camunda cluster; run with `make test-live`"]
async fn ad_hoc_result_activates_exactly_the_chosen_element() {
    let client = client();
    let process_id = unique("ad-hoc");
    let job_type = unique("ad-hoc-worker");
    let chosen = format!("{process_id}-chosen");
    let skipped = format!("{process_id}-skipped");
    let bpmn = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
                  xmlns:zeebe="http://camunda.org/schema/zeebe/1.0"
                  id="defs-{process_id}" targetNamespace="http://bpmn.io/schema/bpmn">
  <bpmn:process id="{process_id}" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:sequenceFlow id="to-ad-hoc" sourceRef="start" targetRef="ad-hoc" />
    <bpmn:adHocSubProcess id="ad-hoc">
      <bpmn:extensionElements>
        <zeebe:taskDefinition type="{job_type}" />
      </bpmn:extensionElements>
      <bpmn:task id="{chosen}" />
      <bpmn:task id="{skipped}" />
    </bpmn:adHocSubProcess>
    <bpmn:sequenceFlow id="to-end" sourceRef="ad-hoc" targetRef="end" />
    <bpmn:endEvent id="end" />
  </bpmn:process>
</bpmn:definitions>
"#
    );
    deploy(&client, &process_id, bpmn).await;

    // The first job chooses the element; the job the engine creates once that element
    // completes fulfils the completion condition.
    let jobs = Arc::new(AtomicUsize::new(0));
    let counted = jobs.clone();
    let element = chosen.clone();
    let handle = worker(&client, &job_type, move |_job| {
        if counted.fetch_add(1, Ordering::SeqCst) == 0 {
            JobAction::complete_with_result(
                AdHocSubProcessResult::new().activate_element(element.as_str()),
            )
        } else {
            JobAction::complete_with_result(
                AdHocSubProcessResult::new().completion_condition_fulfilled(true),
            )
        }
    });

    let key = start(&client, &process_id).await;
    let instance = client
        .eventual_until(
            poll(),
            || client.get_process_instance(&key),
            |instance| instance.state == models::ProcessInstanceStateEnum::Completed,
        )
        .await
        .expect("the process instance completes");
    handle.stop();
    assert_eq!(instance.state, models::ProcessInstanceStateEnum::Completed);

    let elements = element_instances_until(&client, &key, |items| {
        items.iter().any(|e| e.element_id.value() == "end")
    })
    .await;
    let count = |id: &str| {
        elements
            .iter()
            .filter(|e| e.element_id.value() == id)
            .count()
    };
    assert_eq!(
        count(&chosen),
        1,
        "the chosen element ran once: {elements:?}"
    );
    assert_eq!(count(&skipped), 0, "the other element never ran");
    assert_eq!(count("ad-hoc"), 1);
    assert_eq!(
        jobs.load(Ordering::SeqCst),
        2,
        "one job to choose, one to finish"
    );
}
