use argh::FromArgs;
use knot_sys::*;
use std::collections::HashMap;
use std::ffi::CString;

fn normalize(key: &str) -> String {
    key.to_lowercase()
        .replace(' ', "_")
        .replace('/', "_")
        .replace('-', "_")
}

fn parse(value: &str) -> i64 {
    if let Ok(value) = value.parse() {
        return value;
    }
    if let Some(value) = knot_bool_parse(value) {
        return if value { 1 } else { 0 };
    }
    if let Some(value) = knot_time_parse(value) {
        return value as i64;
    }
    -1
}

async fn metrics(req: tide::Request<Args>) -> tide::Result {
    let socket = req.state().socket.clone();
    let buffer = async_std::task::spawn_blocking(
        move || -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
            let ctx = KnotCtx::new();
            ctx.connect(&socket)?;
            ctx.send(
                KnotCtlType::DATA,
                Some(&KnotCtlData::from([(
                    KnotCtlIdx::CMD,
                    CString::new("zone-status")?,
                )])),
            )?;
            ctx.send(KnotCtlType::BLOCK, None)?;
            let registry = prometheus::Registry::new();
            loop {
                let (data_type, data) = ctx.recv()?;
                match data_type {
                    KnotCtlType::BLOCK => break,
                    KnotCtlType::DATA | KnotCtlType::EXTRA => {
                        let zone = data.get(&KnotCtlIdx::ZONE).unwrap().clone().into_string()?;
                        let label = data.get(&KnotCtlIdx::TYPE).unwrap().clone().into_string()?;
                        let value = data.get(&KnotCtlIdx::DATA).unwrap().clone().into_string()?;
                        let gauge = prometheus::IntGauge::with_opts(prometheus::Opts {
                            namespace: "knot".to_string(),
                            subsystem: "dns".to_string(),
                            name: normalize(&label),
                            help: label,
                            const_labels: HashMap::from([("zone".to_string(), zone)]),
                            variable_labels: vec![],
                        })?;
                        gauge.set(parse(&value));
                        registry.register(Box::new(gauge))?;
                    }
                    _ => unreachable!(),
                }
            }
            Ok(prometheus::TextEncoder::new().encode_to_string(&registry.gather())?)
        },
    )
    .await
    .unwrap();
    Ok(buffer.into())
}

#[derive(FromArgs, Clone)]
/// Reach new heights.
struct Args {
    /// knotd control socket path
    #[argh(option, short = 's', default = "String::from(\"/run/knot/knot.sock\")")]
    socket: String,
    /// listen address
    #[argh(option, short = 'l')]
    listen: String,
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    let args: Args = argh::from_env();
    let mut app = tide::with_state(args.clone());
    app.at("metrics").get(metrics);
    app.listen(args.listen).await?;
    Ok(())
}
