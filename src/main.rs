use clap::{AppSettings, Arg, ArgMatches};
use clap_nested::{Command, Commander};
use myco_client_rust::connectors::psubscribe;
use text_colorizer::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {

    Commander::new()
        .options(|app| {
            app.setting(AppSettings::ColoredHelp)
                .arg(
                    Arg::with_name("orderbook-url")
                        .short("u")
                        .long("orderbook-url")
                        .global(true)
                        .takes_value(true)
                        .value_name("STRING")
                        .default_value("redis://127.0.0.1")
                        .help("orderbook url"),
                )
                .arg(
                    Arg::with_name("orderbook-port")
                        .short("p")
                        .long("orderbook-port")
                        .global(true)
                        .takes_value(true)
                        .value_name("STRING")
                        .default_value("6379")
                        .help("orderbook port"),
                )
                .arg(
                    Arg::with_name("settlement-url")
                        .short("U")
                        .long("settlement-url")
                        .global(true)
                        .takes_value(true)
                        .value_name("STRING")
                        .default_value("ws://127.0.0.1")
                        .help("settlement service url"),
                )
                .arg(
                    Arg::with_name("settlement-port")
                        .short("P")
                        .long("settlement-port")
                        .global(true)
                        .takes_value(true)
                        .value_name("STRING")
                        .default_value("9944")
                        .help("settlement service port"),
                )
                .arg(
                    Arg::with_name("telemetry-level")
                        .short("t")
                        .long("telemetry-level")
                        .global(true)
                        .takes_value(true)
                        .value_name("STRING")
                        .default_value("info")
                        .help("telemetry service level"),
                )
                .name("myco-client-rust")
                .version(VERSION)
                .author("Grid Singularity Gmbh <info@gridsingularity.com>")
                .about("Matching engine for the Grid Singularity Energy Exchange")
        })
        .args(|_args, matches| matches.value_of("environment").unwrap_or("dev"))
        .add_cmd(
            Command::new("web2")
                    .description("Connect the matching engine to the web2 Grid Singularity Energy Exchange")
                    .runner(|_args: &str, matches: &ArgMatches<'_>| {
                        let orders_response_channel = String::from("external-myco/*/offers-bids/response/");
                        let recommendations_channel = String::from("external-myco/*/recommendations");
                        let tick_channel = String::from("external-myco/*/events/");

                        let channels = vec![
                            tick_channel.clone(),
                            orders_response_channel.clone(),
                            recommendations_channel.clone()
                        ];

                        let url = format!(
                            "{}:{}",
                            matches.value_of("orderbook-url").unwrap(),
                            matches.value_of("orderbook-port").unwrap()
                        );

                        eprintln!("Connecting to {}", url.green());

                        if let Err(error) = psubscribe(channels.clone(), url) {
                            eprintln!("{} - {:?}", "Error".red().bold(), error);
                            panic!("{:?}", error);
                        } else {
                            println!("subscribed to the following channels:");
                            for channel in channels {
                                eprintln!("{}", channel.green());
                            }
                        }
                        Ok(())
                    }),
        )
        .add_cmd(
            Command::new("web3")
                    .description("Connect the matching engine to the web3 Grid Singularity Energy Exchange")
                    .runner(|_args: &str, _matches: &ArgMatches<'_>| {
                        // TODO: web3 logic (subscribe to block events, fetch orderbook, create extrinsic with match recommandations)
                        Ok(())
                    }),
        )
        .no_cmd(|_args, _matches| {
            eprintln!("No command matched");
            Ok(())
        })
        .run();
}