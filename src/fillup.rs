use std::borrow::Cow;

use rocket::serde::json::{json, Json, Value};
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::sync::Mutex;
use rocket::State;
use rusqlite::{named_params, Connection, Result};

mod mydb {
    use rusqlite::{Connection, Result};

    pub fn initialize_database() -> Result<Connection> {
        let conn = Connection::open("fillup.db")?;

        let result = conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS  cars (
                id TEXT PRIMARY KEY,
                make TEXT NOT NULL,
                model TEXT NOT NULL,
                year INTEGER NOT NULL
            );",
        );
        match result {
            Ok(_) => info!("Table fuel_logs created"),
            Err(e) => info!("Error creating table fuel_logs: {}", e),
        };

        let result = conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS  fuel_logs (
                id INTEGER PRIMARY KEY,
                car_id INTEGER NOT NULL,
                username TEXT NOT NULL,
                miles_driven REAL NOT NULL,
                current_odometer_reading REAL NOT NULL,
                gallons REAL NOT NULL, 
                total_cost REAL,
                cost_per_gallon REAL,
                fill_up_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (car_id) REFERENCES Cars(id)
            );",
        );
        match result {
            Ok(_) => info!("Table fuel_logs created"),
            Err(e) => info!("Error creating table fuel_logs: {}", e),
        };

        info!("Database initialized");

        // Perform any necessary database initialization here
        Ok(conn)
    }

    /*
    pub fn perform_database_query(conn: &Connection) -> Result<()> {
        // Perform database query using the provided connection
        // For example:
        conn.execute("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT)", [])?;
        Ok(())
    }
    */
}
// The type to represent the ID of a message.
type Id = usize;

// We're going to store all of the messages here. No need for a DB.
type MessageList = Mutex<Vec<String>>;
type Messages<'r> = &'r State<MessageList>;

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Message<'r> {
    id: Option<Id>,
    message: Cow<'r, str>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]

struct Car {
    id: String,
    make: Option<String>,
    model: Option<String>,
    year: i32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
struct FuelLog<'r> {
    id: Option<Id>,
    car_id: String,
    username: Cow<'r, String>,
    miles_driven: f64,
    current_odometer_reading: f64,
    gallons: f64,
    total_cost: Option<f64>,
    cost_per_gallon: Option<f64>,
    fill_up_date: Option<String>,
}

#[post("/", format = "json", data = "<message>")]
async fn fillup(message: Json<FuelLog<'_>>, list: Messages<'_>) -> Value {
    let mut list = list.lock().await;
    let id = list.len();

    let fuel_log: FuelLog = message.into_inner();
    info!("{:#?}", fuel_log);
    fuel_log
        .add_to_database(&mydb::initialize_database().unwrap())
        .unwrap();
    let json_string = serde_json::to_string(&fuel_log).expect("Failed to serialize JSON");

    list.push(json_string);
    json!({ "status": "ok", "id": id })
}

#[post("/newcar", format = "json", data = "<car>")]
async fn new_car(car: Json<Car>) -> Value {
    //let mut list = list.lock().await;
    //let id = list.len();

    let car: Car = car.into_inner();
    info!("{:#?}", car);
    car.add_to_database(&mydb::initialize_database().unwrap())
        .unwrap();
    //let json_string = serde_json::to_string(&car).expect("Failed to serialize JSON");

    //list.push(json_string);
    json!({ "status": "ok", "id": car.id })
}

#[put("/<id>", format = "json", data = "<message>")]
async fn update(id: Id, message: Json<Message<'_>>, list: Messages<'_>) -> Option<Value> {
    match list.lock().await.get_mut(id) {
        Some(existing) => {
            *existing = message.message.to_string();
            Some(json!({ "status": "ok" }))
        }
        None => None,
    }
}

/*
#[get("/cars/all", format = "json")]
async fn request_all_cars() -> Option<Value> {

    let list = car::get_all_from_database(&mydb::initialize_database().unwrap());
    match list {
        Ok(list) => json!({ "status": "ok", "list": list }),
        Err(e) => json!({ "status": "error", "error": e.to_string() })
    }

}
*/

#[get("/cars/<id>", format = "json")]
async fn get_car(id: String, list: Messages<'_>) -> Option<Value> {
    let list = list.lock().await;
    list.iter()
        .find(|car| car.to_string() == id.to_string())
        .map(|car| json!({ "status": "ok", "car": car }))
}

#[get("/<id>", format = "json")]
async fn get(id: Id, list: Messages<'_>) -> Option<Json<Message<'_>>> {
    let list = list.lock().await;

    Some(Json(Message {
        id: Some(id),
        message: list.get(id)?.to_string().into(),
    }))
}

#[catch(404)]
fn not_found() -> Value {
    json!({
        "status": "error",
        "reason": "Resource was not found."
    })
}

pub fn stage() -> rocket::fairing::AdHoc {
    rocket::fairing::AdHoc::on_ignite("JSON", |rocket| async {
        rocket
            .mount(
                "/fillup/data",
                routes![new_car, update, get, fillup, get_car],
            )
            .register("/fillup/data", catchers![not_found])
            .manage(MessageList::new(vec![]))
    })
}

impl Car {
    fn add_to_database(&self, conn: &Connection) -> Result<&str> {
        conn.execute(
            "insert into cars(id , make, model, year ) 
        VALUES (:id , :make, :model, :year);",
            named_params! {
                ":id " : self.id,
                ":make " : self.make,
                ":model  " : self.model,
                ":year " : self.year
            },
        )?;
        Ok(self.id.as_str())
    }

    fn get_all_from_database(&self, conn: &Connection) -> Result<Vec<Car>> {
        let mut stmt = conn.prepare("SELECT id, make,  model, year FROM cars  ")?;
        let car_iter = stmt.query_map([], |row| {
            Ok(Car {
                id: row.get(0)?,
                make: row.get(1)?,
                model: row.get(2)?,
                year: row.get(3)?,
            })
        })?;
        let mut car_list = Vec::new();
        for car in car_iter {
            car_list.push(car.unwrap());
        }

        Ok(car_list)
    }
}

impl FuelLog<'_> {
    fn add_to_database(&self, conn: &Connection) -> Result<i64> {
        conn.execute(
            "INSERT INTO fuel_logs (car_id, username, miles_driven,
            current_odometer_reading,
            gallons, 
            total_cost,
            cost_per_gallon   ) VALUES    
         (:car_id, :username, :miles_driven, :current_odometer_reading,
            :gallons, 
            :total_cost,
            :cost_per_gallon )",
            named_params! {
                ":car_id": self.car_id,
                ":username": self.username,
                ":miles_driven": self.miles_driven,
                ":current_odometer_reading": self.current_odometer_reading,
                ":gallons": self.gallons,
                ":total_cost": self.total_cost,
                ":cost_per_gallon": self.cost_per_gallon
            },
        )?;
        Ok(conn.last_insert_rowid())
    }
}
