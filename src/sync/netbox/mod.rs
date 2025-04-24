pub(super) mod model;

use std::collections::HashMap;

use chrono::NaiveDate;
use ipnet::Ipv4Net;
use log::debug;
use serde::Deserialize;
use serde_json::json;
use ureq::http::{HeaderValue, Request};
use ureq::tls::{RootCerts, TlsConfig};
use ureq::{Agent, Body, SendBody};
use ureq::{middleware::MiddlewareNext, http::response::Response, Error};

pub mod config;
use self::config::SyncNetboxConfig;
use self::model::*;
pub mod prefix;
use prefix::*;
pub mod range;
use range::*;
pub mod address;
use address::*;

pub struct NetboxApi {
    config: SyncNetboxConfig,
    client: ureq::Agent,
}

impl NetboxApi {
    pub fn new(config: &SyncNetboxConfig) -> Self {
        let config = config.clone();

        let auth_value = format!("Token {}", config.token());

        let client = Agent::config_builder()
            .tls_config(
                TlsConfig::builder()
                    .root_certs(RootCerts::PlatformVerifier)
                    .build()
            )
            .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
            .middleware(move |mut req: Request<SendBody>, next: MiddlewareNext| -> Result<Response<Body>, Error> {
                req.headers_mut().append("Authorization", HeaderValue::from_str(auth_value.as_str()).unwrap());
                next.handle(req)
            })
            .build()
            .into();

        Self { config, client }
    }

    pub fn version(
        &self,
    ) -> Result<String, Box<dyn std::error::Error + Send + std::marker::Sync>> {
        let url = format!("{}status/", self.config.apiurl());

        #[derive(Debug, Deserialize)]
        struct NetboxStatus {
            #[serde(rename = "netbox-version")]
            netbox_version: String,
        }

        let status: NetboxStatus = self.client.get(&url)
            .call()?
            .body_mut()
            .read_json()?;

        Ok(status.netbox_version)
    }

    pub fn get_prefixes(&self) -> Result<Vec<Prefix>, ureq::Error> {
        self.get_objects("ipam/prefixes/", self.config.prefix_filter())
    }

    pub fn get_ranges(&self) -> Result<Vec<IpRange>, ureq::Error> {
        self.get_objects("ipam/ip-ranges/", self.config.range_filter())
    }

    pub fn get_reservations(&self) -> Result<Vec<IpAddress>, ureq::Error> {
        self.get_objects("ipam/ip-addresses/", &self.config.reservation_filter(None))
    }

    pub fn get_reservations_for_subnet(&self, subnet: &Ipv4Net) -> Result<Vec<IpAddress>, ureq::Error> {
        self.get_objects("ipam/ip-addresses/", &self.config.reservation_filter(Some(subnet)))
    }

    pub fn get_router_for_subnet(&self, subnet: &Ipv4Net) -> Result<Vec<IpAddress>, ureq::Error> {
        self.get_objects("ipam/ip-addresses/", &self.config.router_filter(subnet))
    }

    pub fn set_ip_last_active(&self, ip: &IpAddress, date: &NaiveDate) -> Result<(), ureq::Error> {
        let payload = json!({
            "custom_fields": {
                "dhcp_reservation_last_active": date,
            }
        });

        self.client.patch(ip.url())
            .header("Content-Type", "application/json")
            .send(payload.to_string().as_str())?;

        Ok(())
    }

    fn get_objects<T: for<'a> Deserialize<'a>>(
        &self,
        path: &str,
        filter: &HashMap<String, String>,
    ) -> Result<Vec<T>, ureq::Error> {
        let url = format!("{}{}", self.config.apiurl(), path);

        let mut query: Vec<(&str, &str)> = vec![];
        for (key, val) in filter.iter() {
            query.push((key.as_str(), val.as_str()));
        }

        debug!("Fetch {} from {:?}", std::any::type_name::<T>(), url);
        let mut page: Pageination<T> = self.client.get(&url)
            .query_pairs(query)
            .call()?
            .body_mut()
            .read_json()?;

        let mut objects: Vec<T> = Vec::with_capacity(page.count);

        loop {
            objects.append(&mut page.results);

            match page.next {
                Some(ref u) => {
                    debug!("Fetch next page from {:?}", u);
                    page = self.client.get(u)
                        .call()?
                        .body_mut()
                        .read_json()?;
                }
                None => { break; }
            }
        }

        Ok(objects)
    }

    pub fn get_object<T: for<'a> Deserialize<'a>>(&self, url: &str) -> Result<T, ureq::Error> {
        debug!("Fetch {} from {:?}", std::any::type_name::<T>(), url);
        let object: T = self.client.get(url)
            .call()?
            .body_mut()
            .read_json()?;
        Ok(object)
    }
}
