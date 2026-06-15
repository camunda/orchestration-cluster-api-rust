//! The ergonomic [`CamundaClient`] facade over the generated low-level client.

use std::collections::HashMap;
use std::sync::Arc;

use camunda_orchestration_api_client::apis::configuration::Configuration;
use camunda_orchestration_api_client::apis::{
    cluster_api, job_api, process_instance_api, resource_api,
};
use camunda_orchestration_api_client::models;

use super::auth::Authentication;
use super::backpressure::{is_backpressure_error, BackpressureManager, BackpressureState};
use super::config::CamundaConfig;
use super::errors::Result;
use super::job_worker::{JobWorker, JobWorkerConfig};

/// Options for constructing a [`CamundaClient`].
#[derive(Default)]
pub struct CamundaOptions {
    /// Programmatic overrides for `CAMUNDA_*` configuration keys. These take precedence
    /// over environment variables.
    pub config: HashMap<String, String>,
    /// A pre-built `reqwest::Client` to use for all requests (including OAuth token
    /// fetches). When `None`, a default client is created.
    pub http_client: Option<reqwest::Client>,
}

impl CamundaOptions {
    /// Create empty options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a single `CAMUNDA_*` configuration override.
    pub fn with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.insert(key.into(), value.into());
        self
    }

    /// Use a custom `reqwest::Client`.
    pub fn with_http_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = Some(client);
        self
    }
}

/// The primary entry point of the SDK.
///
/// A `CamundaClient` is cheap to clone — clones share the same configuration,
/// HTTP client, and OAuth token cache.
#[derive(Clone)]
pub struct CamundaClient {
    config: CamundaConfig,
    auth: Authentication,
    http: reqwest::Client,
    user_agent: String,
    bp: Arc<BackpressureManager>,
}

impl CamundaClient {
    /// Construct a client from environment variables only.
    pub fn from_env() -> Result<Self> {
        Self::new(CamundaOptions::new())
    }

    /// Construct a client from [`CamundaOptions`] (environment + overrides).
    pub fn new(options: CamundaOptions) -> Result<Self> {
        let config = CamundaConfig::from_env_with_overrides(&options.config)?;
        let http = options.http_client.unwrap_or_default();
        let auth = Authentication::from_config(&config, http.clone());
        let bp = Arc::new(BackpressureManager::new(config.backpressure_profile));
        Ok(CamundaClient {
            config,
            auth,
            http,
            user_agent: format!(
                "camunda-orchestration-sdk-rust/{}",
                env!("CARGO_PKG_VERSION")
            ),
            bp,
        })
    }

    /// The resolved configuration.
    pub fn config(&self) -> &CamundaConfig {
        &self.config
    }

    /// The authentication handler.
    pub fn auth(&self) -> &Authentication {
        &self.auth
    }

    /// A snapshot of the adaptive backpressure controller's state, for observability.
    pub fn backpressure_state(&self) -> BackpressureState {
        self.bp.state()
    }

    /// Run an initiating operation under the adaptive backpressure gate: acquire a permit,
    /// await the operation, then record the outcome and release the permit.
    ///
    /// Drain operations (job completion / failure / BPMN error) bypass this gate so that
    /// in-flight work can always be drained even while the cluster is shedding new load.
    async fn guarded<T, F, Fut>(&self, op: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        self.bp.acquire().await?;
        let result = op().await;
        match &result {
            Ok(_) => self.bp.record_healthy_hint(),
            Err(e) if is_backpressure_error(e) => self.bp.record_backpressure(),
            Err(_) => {}
        }
        self.bp.release();
        result
    }

    /// Build a generated-client [`Configuration`] with the base URL set and the current
    /// authentication applied (refreshing the OAuth token if needed).
    ///
    /// Use this to call generated operations that the facade does not yet wrap:
    ///
    /// ```no_run
    /// # use camunda_orchestration_sdk::CamundaClient;
    /// use camunda_orchestration_sdk::client::apis::authentication_api;
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = CamundaClient::from_env()?;
    /// let cfg = client.configuration().await?;
    /// let me = authentication_api::get_authentication(&cfg).await?;
    /// # let _ = me;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn configuration(&self) -> Result<Configuration> {
        let mut cfg = Configuration {
            base_path: self.config.rest_address.clone(),
            user_agent: Some(self.user_agent.clone()),
            client: self.http.clone(),
            basic_auth: None,
            oauth_access_token: None,
            bearer_access_token: None,
            api_key: None,
        };
        self.auth.apply(&mut cfg).await?;
        Ok(cfg)
    }

    // --- Ergonomic operation wrappers ---------------------------------------

    /// Fetch the cluster topology.
    pub async fn topology(&self) -> Result<models::TopologyResponse> {
        self.guarded(|| async {
            let cfg = self.configuration().await?;
            Ok(cluster_api::get_topology(&cfg).await?)
        })
        .await
    }

    /// Create (start) a process instance.
    pub async fn create_process_instance(
        &self,
        instruction: models::ProcessInstanceCreationInstruction,
    ) -> Result<models::CreateProcessInstanceResult> {
        self.guarded(|| async {
            let cfg = self.configuration().await?;
            let params = process_instance_api::CreateProcessInstanceParams {
                process_instance_creation_instruction: instruction,
            };
            Ok(process_instance_api::create_process_instance(&cfg, params).await?)
        })
        .await
    }

    /// Deploy one or more resources (BPMN, DMN, forms) from local file paths.
    pub async fn deploy_resources(
        &self,
        resources: Vec<std::path::PathBuf>,
        tenant_id: Option<String>,
    ) -> Result<models::DeploymentResult> {
        self.guarded(|| async {
            let cfg = self.configuration().await?;
            let params = resource_api::CreateDeploymentParams {
                resources,
                tenant_id,
            };
            Ok(resource_api::create_deployment(&cfg, params).await?)
        })
        .await
    }

    // --- Job operations (also used by the job worker) -----------------------

    /// Activate jobs of a given type. Prefer [`CamundaClient::create_job_worker`] for
    /// continuous polling.
    pub async fn activate_jobs(
        &self,
        request: models::JobActivationRequest,
    ) -> Result<models::JobActivationResult> {
        self.guarded(|| async {
            let cfg = self.configuration().await?;
            let params = job_api::ActivateJobsParams {
                job_activation_request: request,
            };
            Ok(job_api::activate_jobs(&cfg, params).await?)
        })
        .await
    }

    /// Complete a job, optionally with output variables.
    ///
    /// Job completion is a *drain* operation and intentionally bypasses the backpressure
    /// gate so in-flight work can always be drained, even while new load is being shed.
    pub async fn complete_job(
        &self,
        job_key: &str,
        request: Option<models::JobCompletionRequest>,
    ) -> Result<()> {
        let cfg = self.configuration().await?;
        let params = job_api::CompleteJobParams {
            job_key: job_key.to_string(),
            job_completion_request: request,
        };
        Ok(job_api::complete_job(&cfg, params).await?)
    }

    /// Fail a job, decrementing retries.
    pub async fn fail_job(
        &self,
        job_key: &str,
        request: Option<models::JobFailRequest>,
    ) -> Result<()> {
        let cfg = self.configuration().await?;
        let params = job_api::FailJobParams {
            job_key: job_key.to_string(),
            job_fail_request: request,
        };
        Ok(job_api::fail_job(&cfg, params).await?)
    }

    /// Throw a BPMN error from a job.
    pub async fn throw_job_error(
        &self,
        job_key: &str,
        request: models::JobErrorRequest,
    ) -> Result<()> {
        let cfg = self.configuration().await?;
        let params = job_api::ThrowJobErrorParams {
            job_key: job_key.to_string(),
            job_error_request: request,
        };
        Ok(job_api::throw_job_error(&cfg, params).await?)
    }

    /// Create a [`JobWorker`] that continuously polls for and processes jobs of the
    /// configured type. Call [`JobWorker::run`] (or [`JobWorker::start`]) to begin.
    pub fn create_job_worker(&self, config: JobWorkerConfig) -> JobWorker {
        JobWorker::new(self.clone(), config)
    }
}
