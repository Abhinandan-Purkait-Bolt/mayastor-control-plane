//! The Core Agent.
//! todo: document.

/// The controller logic for all resources.
pub(crate) mod controller;
/// The nexus related operations.
pub(crate) mod nexus;
/// The node related operations.
pub(crate) mod node;
/// The pool related operations.
pub(crate) mod pool;
/// The registry which contains all the resources.
pub(crate) mod registry;
/// The volume related operations.
pub(crate) mod volume;
/// The watch related operations.
pub(crate) mod watch;

use controller::registry::NumRebuilds;
use std::net::SocketAddr;
use utils::{version_info_str, DEFAULT_GRPC_SERVER_ADDR};

use common_lib::HostAccessControl;
use opentelemetry::{trace::TracerProvider, KeyValue};
use structopt::StructOpt;

/// The Cli arguments for this binary.
#[derive(Debug, StructOpt)]
#[structopt(name = utils::package_description!(), version = version_info_str!())]
pub(crate) struct CliArgs {
    /// The period at which the registry updates its cache of all
    /// resources from all nodes.
    #[structopt(long, short, default_value = utils::CACHE_POLL_PERIOD)]
    pub(crate) cache_period: humantime::Duration,

    /// The period at which the reconcile loop checks for new work.
    #[structopt(long, default_value = "30s")]
    pub(crate) reconcile_idle_period: humantime::Duration,

    /// The period at which the reconcile loop attempts to do work.
    #[structopt(long, default_value = "10s")]
    pub(crate) reconcile_period: humantime::Duration,

    /// Deadline for the io-engine instance keep alive registration.
    #[structopt(long, short, default_value = "10s")]
    pub(crate) deadline: humantime::Duration,

    /// The Persistent Store URLs to connect to.
    /// (supports the http/https schema)
    #[structopt(long, short, default_value = "http://localhost:2379")]
    pub(crate) store: String,

    /// The timeout for store operations.
    #[structopt(long, default_value = utils::STORE_OP_TIMEOUT)]
    pub(crate) store_timeout: humantime::Duration,

    /// The lease lock ttl for the persistent store after which we'll lose the exclusive access.
    #[structopt(long, default_value = utils::STORE_LEASE_LOCK_TTL)]
    pub(crate) store_lease_ttl: humantime::Duration,

    /// The timeout for every node connection (gRPC).
    #[structopt(long, default_value = utils::DEFAULT_CONN_TIMEOUT)]
    pub(crate) connect_timeout: humantime::Duration,

    /// The default timeout for node request timeouts (gRPC).
    #[structopt(long, short, default_value = utils::DEFAULT_REQ_TIMEOUT)]
    pub(crate) request_timeout: humantime::Duration,

    /// Control hosts access control via their NQN's.
    #[structopt(long, short, use_delimiter = true, default_value = utils::DEFAULT_HOST_ACCESS_CONTROL)]
    pub(crate) hosts_acl: Vec<HostAccessControl>,

    /// Add process service tags to the traces.
    #[structopt(short, long, env = "TRACING_TAGS", value_delimiter=",", parse(try_from_str = utils::tracing_telemetry::parse_key_value))]
    tracing_tags: Vec<KeyValue>,

    /// Don't use minimum timeouts for specific requests.
    #[structopt(long)]
    no_min_timeouts: bool,
    /// Trace rest requests to the Jaeger endpoint agent.
    #[structopt(long, short)]
    jaeger: Option<String>,
    /// The GRPC Server URLs to connect to.
    /// (supports the http/https schema)
    #[structopt(long, short, default_value = DEFAULT_GRPC_SERVER_ADDR)]
    pub(crate) grpc_server_addr: SocketAddr,
    /// The maximum number of system-wide rebuilds permitted at any given time.
    /// If `None` do not limit the number of rebuilds.
    #[structopt(long)]
    max_rebuilds: Option<NumRebuilds>,
}
impl CliArgs {
    fn args() -> Self {
        CliArgs::from_args()
    }
}

#[tokio::main]
async fn main() {
    let cli_args = CliArgs::args();
    utils::print_package_info!();
    println!("Using options: {:?}", &cli_args);
    utils::tracing_telemetry::init_tracing(
        "core-agent",
        cli_args.tracing_tags.clone(),
        cli_args.jaeger.clone(),
    );
    server(cli_args).await;
}

async fn server(cli_args: CliArgs) {
    common_lib::init_cluster_info_or_panic().await;
    let registry = controller::registry::Registry::new(
        cli_args.cache_period.into(),
        cli_args.store.clone(),
        cli_args.store_timeout.into(),
        cli_args.store_lease_ttl.into(),
        cli_args.reconcile_period.into(),
        cli_args.reconcile_idle_period.into(),
        cli_args.max_rebuilds,
        if cli_args.hosts_acl.contains(&HostAccessControl::None) {
            vec![]
        } else {
            cli_args.hosts_acl
        },
    )
    .await;

    let service = agents::Service::builder()
        .with_shared_state(opentelemetry::global::tracer_provider().versioned_tracer(
            "core-agent",
            Some(env!("CARGO_PKG_VERSION")),
            None,
        ))
        .with_shared_state(registry.clone())
        .with_shared_state(cli_args.grpc_server_addr)
        .configure_async(node::configure)
        .await
        .configure(pool::configure)
        .configure(nexus::configure)
        .configure(volume::configure)
        .configure(watch::configure)
        .configure(registry::configure);

    registry.start().await;
    service.run(cli_args.grpc_server_addr).await;
    registry.stop().await;
    opentelemetry::global::shutdown_tracer_provider();
}
