use chrono::{SecondsFormat, Utc};
use reqwest::{Client, StatusCode};
use std::env;

pub mod facets;
pub mod structs;

use structs::{
    AuthenticationBody, AuthenticationResponse, BskyError, Embed, Error, Facet, FeedListOptions,
    FeedResponse, FollowersListOptions, FollowsResponse, NewPost, NewRecord, PostRef, RecordType,
    Reply,
};

#[derive(Clone)]
pub struct Bsky {
    access_jwt: Option<String>,
    refresh_jwt: Option<String>,
    handle: Option<String>,
    did: Option<String>,
    client: Client,
}

impl Bsky {
    pub fn new() -> Self {
        let client = reqwest::Client::new();
        Bsky {
            access_jwt: None,
            refresh_jwt: None,
            handle: None,
            did: None,
            client,
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.access_jwt.is_some()
    }

    pub async fn authenticate(&mut self) -> Result<(), Error> {
        let bsky_handle = env::var("BLUESKY_HANDLE").expect("BLUESKY_HANDLE must be set.");
        let bsky_pass = env::var("BLUESKY_PASSWORD").expect("BLUESKY_PASSWORD must be set.");

        let body = AuthenticationBody {
            identifier: bsky_handle,
            password: bsky_pass,
        };

        let res = self
            .client
            .post("https://bsky.social/xrpc/com.atproto.server.createSession")
            .json(&body)
            .send()
            .await;

        if res.is_ok() {
            let res = res.unwrap();
            let status = res.status();

            if status != StatusCode::OK {
                return Err(Error::BskyError(res.json::<BskyError>().await.unwrap()));
            }

            let res = res.json::<AuthenticationResponse>().await.unwrap();

            if res.status.is_some() {
                panic!("Unexpected issue with account: {}", res.status.unwrap())
            }

            self.access_jwt = Some(res.access_jwt);
            self.refresh_jwt = Some(res.refresh_jwt);
            self.handle = Some(res.handle);
            self.did = Some(res.did);

            Ok(())
        } else {
            Err(Error::ReqwestError(res.err().unwrap()))
        }
    }

    // pub async fn refresh(&mut self) -> Result<(), Error> {
    //     let token = self
    //         .refresh_jwt
    //         .as_ref()
    //         .expect("Refresh Token was not initialized");
    //     let res = self
    //         .client
    //         .post("https://public.api.bsky.app/xrpc/com.atproto.server.refreshSession")
    //         .bearer_auth(&token)
    //         .send()
    //         .await;

    //     if res.is_ok() {
    //         let res = res.unwrap();
    //         let status = res.status();

    //         if status != StatusCode::OK {
    //             return Err(Error::BskyError(res.json::<BskyError>().await.unwrap()));
    //         }

    //         let res = res.json::<AuthenticationResponse>().await.unwrap();

    //         if res.status.is_some() {
    //             panic!("Unexpected issue with account: {}", res.status.unwrap())
    //         }

    //         self.access_jwt = Some(res.access_jwt);
    //         self.refresh_jwt = Some(res.refresh_jwt);
    //         self.handle = Some(res.handle);
    //         self.did = Some(res.did);

    //         Ok(())
    //     } else {
    //         self.authenticate().await
    //     }
    // }

    pub async fn get_actor_followers(
        &self,
        options: &FollowersListOptions,
    ) -> Result<FollowsResponse, Error> {
        let res = self
            .client
            .get("https://public.api.bsky.app/xrpc/app.bsky.graph.getFollows")
            .query(&options)
            .send()
            .await;

        if res.is_ok() {
            let res = res.unwrap();
            let status = res.status();

            if status != StatusCode::OK {
                return Err(Error::BskyError(res.json::<BskyError>().await.unwrap()));
            }

            let res = res.json::<FollowsResponse>().await.unwrap();

            Ok(res)
        } else {
            Err(Error::ReqwestError(res.err().unwrap()))
        }
    }

    pub async fn get_author_feed(&self, options: &FeedListOptions) -> Result<FeedResponse, Error> {
        let res = self
            .client
            .get("https://public.api.bsky.app/xrpc/app.bsky.feed.getAuthorFeed")
            .query(&options)
            .send()
            .await;

        if res.is_ok() {
            let res = res.unwrap();
            let status = res.status();

            if status != StatusCode::OK {
                return Err(Error::BskyError(res.json::<BskyError>().await.unwrap()));
            }

            let res = res.json::<FeedResponse>().await.unwrap();

            Ok(res)
        } else {
            Err(Error::ReqwestError(res.err().unwrap()))
        }
    }

    pub async fn create_post(
        &self,
        message: String,
        facets: Option<Vec<Facet>>,
        reply: Option<Reply>,
        embed: Option<Embed>,
    ) -> Result<PostRef, Error> {
        let did = self.did.clone().expect("Session was not initialized");

        let test_post = NewPost {
            repo: did,
            collection: RecordType::Post,
            lang: vec![String::from("ua")],
            validate: true,
            record: NewRecord {
                record_type: RecordType::Post,
                text: message,
                created_at: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
                facets,
                reply,
                embed,
            },
        };

        let token = self
            .access_jwt
            .as_ref()
            .expect("Access Token was not initialized");

        let res = self
            .client
            .post("https://bsky.social/xrpc/com.atproto.repo.createRecord")
            .bearer_auth(&token)
            .json(&test_post)
            .send()
            .await;

        if res.is_ok() {
            let res = res.unwrap();
            let status = res.status();

            if status != StatusCode::OK {
                return Err(Error::BskyError(res.json::<BskyError>().await.unwrap()));
            }

            let res = res.json::<PostRef>().await.unwrap();

            Ok(res)
        } else {
            Err(Error::ReqwestError(res.err().unwrap()))
        }
    }
}
