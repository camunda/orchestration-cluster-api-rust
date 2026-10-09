//! Hand-written runtime: configuration, authentication, errors, the ergonomic
//! [`client::CamundaClient`] facade, and [`job_worker::JobWorker`].

pub mod auth;
pub mod backpressure;
pub mod client;
pub mod clock;
pub mod config;
pub mod errors;
pub mod eventual;
pub mod facade_generated;
pub mod falcon;
pub mod job_worker;
pub mod logging;
mod present_when;
mod present_when_generated;
pub mod random;
pub mod retry;
pub mod tls;
