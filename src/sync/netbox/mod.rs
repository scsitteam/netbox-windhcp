pub(super) mod model;

use std::collections::HashMap;

use ipnet::Ipv4Net;
use log::debug;
use serde::Deserialize;

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
    client: reqwest::Client,
}

impl NetboxApi {
    pub fn new(config: &SyncNetboxConfig) -> Self {
        let config = config.clone();

        let mut headers = reqwest::header::HeaderMap::new();
        let mut auth_value = reqwest::header::HeaderValue::from_str(format!("Token {}", config.token()).as_str()).unwrap();
        auth_value.set_sensitive(true);
        headers.insert(reqwest::header::AUTHORIZATION, auth_value);

        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .default_headers(headers)
            .build().unwrap();

        Self { config, client }
    }

    pub async fn version(&self) -> Result<String, Box<dyn std::error::Error + Send + std::marker::Sync>> {
        let url = format!("{}status/", self.config.apiurl());

        #[derive(Debug, Deserialize)]
        struct NetboxStatus {
            #[serde(rename="netbox-version")]
            netbox_version: String
        }

        let status: NetboxStatus = self.client.get(url)
            .send().await?
            .error_for_status()?
            .json().await?;

        Ok(status.netbox_version)
    }

    pub async fn get_prefixes(&self) -> reqwest::Result<Vec<Prefix>> {
        self.get_objects("ipam/prefixes/", self.config.prefix_filter()).await
    }

    pub async fn get_ranges(&self) -> reqwest::Result<Vec<IpRange>> {
        self.get_objects("ipam/ip-ranges/", self.config.range_filter()).await
    }

    pub async fn get_reservations_for_subnet(&self, subnet: &Ipv4Net) -> reqwest::Result<Vec<IpAddress>> {
        self.get_objects("ipam/ip-addresses/", &self.config.reservation_filter(subnet)).await
    }

    pub async fn get_router_for_subnet(&self, subnet: &Ipv4Net) -> reqwest::Result<Vec<IpAddress>> {
        self.get_objects("ipam/ip-addresses/", &self.config.router_filter(subnet)).await
    }

    async fn get_objects<T: for<'a> Deserialize<'a>>(&self, path: &str, filter: &HashMap<String, String>) -> reqwest::Result<Vec<T>> {
        let url = format!("{}{}", self.config.apiurl(), path);

        debug!("Fetch {} from {:?}", std::any::type_name::<T>(), url);
        let mut page: Pageination<T> = self.client.get(url)
            .query(filter)
            .send().await?
            .error_for_status()?
            .json().await?;

        let mut objects: Vec<T> = Vec::with_capacity(page.count);

        loop {
            objects.append(&mut page.results);
    
            match page.next {
                Some(ref u) => {
                    debug!("Fetch next page from {:?}", u);
                    page = self.client.get(u)
                        .send().await?
                        .error_for_status()?
                        .json().await?;
                }
                None => { break; }
            }
        }

        Ok(objects)
    }

    pub async fn get_object<T: for<'a> Deserialize<'a>>(&self, url: &str) -> reqwest::Result<T> {
        debug!("Fetch {} from {:?}", std::any::type_name::<T>(), url);
        let object: T = self.client.get(url)
            .send().await?
            .error_for_status()?
            .json().await?;

        Ok(object)
    }
}