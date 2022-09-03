use actix_web::{
    web::{self, Data},
    App, HttpServer,
};
use mongodb::Client;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let client = Client::with_uri_str("mongodb://root:example@localhost:27017")
        .await
        .unwrap();
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(client.database("otus")))
            .service(web::resource("/users").route(web::post().to(users::web::save_new)))
            .service(web::resource("/users/{username}").route(web::get().to(users::web::find)))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

mod users {
    use mongodb::bson::Bson;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, PartialEq, Eq, Deserialize, Serialize)]
    pub struct NewUser {
        pub username: String,
        pub email: String,
    }

    #[derive(Clone, PartialEq, Deserialize, Serialize)]
    pub struct User {
        pub _id: Bson,
        pub username: String,
        pub email: String,
    }

    pub mod web {
        use super::NewUser;
        use super::User;
        use actix_web::{
            web::{Data, Json, Path},
            HttpResponse,
        };
        use mongodb::Database;

        pub async fn save_new(mongo: Data<Database>, new_user: Json<NewUser>) -> HttpResponse {
            match new_user.into_inner().save(&mongo).await {
                Ok(()) => HttpResponse::Ok().json(()),
                Err(err) => HttpResponse::InternalServerError().json(err),
            }
        }

        pub async fn find(mongo: Data<Database>, username: Path<String>) -> HttpResponse {
            match User::find(username.into_inner().as_str(), &mongo).await {
                Ok(user) => HttpResponse::Ok().json(user),
                Err(err) => HttpResponse::InternalServerError().json(err),
            }
        }
    }

    mod db {
        use super::{NewUser, User};
        use mongodb::{bson::doc, Database};

        const USERS_COLLECTION: &str = "users";

        impl NewUser {
            pub async fn save(&self, mongo: &Database) -> Result<(), String> {
                mongo
                    .collection::<NewUser>(USERS_COLLECTION)
                    .insert_one(self, None)
                    .await
                    .map_err(|err| format!("DB_ERROR: {:?}", err))?;
                Ok(())
            }
        }

        impl User {
            pub async fn find(username: &str, mongo: &Database) -> Result<Self, String> {
                match mongo
                    .collection::<User>(USERS_COLLECTION)
                    .find_one(doc! {"username": username}, None)
                    .await
                {
                    Ok(Some(user)) => Ok(user),
                    Ok(None) => Err(format!("USER_NOT_FOUND: {}", username)),
                    Err(err) => Err(format!("DB_ERROR: {:?}", err)),
                }
            }
        }
    }
}
