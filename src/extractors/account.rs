use actix_web::error::{ErrorBadRequest, ErrorUnauthorized};
use actix_web::{dev, web, Error, FromRequest, HttpRequest};
use futures::future::{err, ok, Ready};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

use crate::models::account::AccountClaims;
use crate::state::AppState;

pub struct AuthorizationGuard(pub i32);

impl FromRequest for AuthorizationGuard {
    type Error = Error;
    type Future = Ready<Result<AuthorizationGuard, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        // let auth = req.headers().get("Authorization");
        // match auth {
        //     Some(header) => {
        //         let state: &web::Data<AppState> = req
        //             .app_data()
        //             .expect("AppState None in AuthorizationGuard, this should never happen");
        //         let token = header.to_str().map_err(ErrorBadRequest).unwrap();
        //         let token_data = decode::<AccountClaims>(
        //             &token[7..],
        //             &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        //             &Validation::new(Algorithm::HS256),
        //         );
        //         match token_data {
        //             Ok(data) => ok(Self(data.claims.sub.parse().unwrap())),
        //             Err(_) => err(ErrorUnauthorized("")),
        //         }
        //     }
        //     None => err(ErrorUnauthorized("")),
        // }
        ok(Self(1)) // 临时返回一个固定的 account_id
    }
}
