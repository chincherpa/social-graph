use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Person {
    pub id: i64,
    pub first_name: Option<String>,
    pub last_name: String,
    pub nickname: Option<String>,
    pub birth_date: Option<String>,
    pub known_since: Option<String>,
    pub address: Option<String>,
    pub employer: Option<String>,
    pub note: Option<String>,
    pub color: String,
    pub gender: Option<String>,
    pub image_path: Option<String>,
    pub image_crop_x: Option<i64>,
    pub image_crop_y: Option<i64>,
    pub image_crop_radius: Option<i64>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub geocoded_at: Option<String>,
    pub last_contact_type: Option<String>,
    pub last_contact_date: Option<String>,
    pub created_at: String,
    #[sqlx(skip)]
    pub image_data: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NewPerson {
    pub first_name: Option<String>,
    pub last_name: String,
    pub nickname: Option<String>,
    pub birth_date: Option<String>,
    pub known_since: Option<String>,
    pub address: Option<String>,
    pub employer: Option<String>,
    pub note: Option<String>,
    pub color: Option<String>,
    pub gender: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePerson {
    pub id: i64,
    pub first_name: Option<String>,
    pub last_name: String,
    pub nickname: Option<String>,
    pub birth_date: Option<String>,
    pub known_since: Option<String>,
    pub address: Option<String>,
    pub employer: Option<String>,
    pub note: Option<String>,
    pub color: String,
    pub gender: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Relationship {
    pub id: i64,
    pub from_id: i64,
    pub to_id: i64,
    pub kind: String,
    pub strength: i64,
    pub note: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct NewRelationship {
    pub person_a: i64,
    pub person_b: i64,
    pub kind: String,
    pub strength: i64,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRelationship {
    pub id: i64,
    pub kind: String,
    pub strength: i64,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct FamilyMember {
    pub id: i64,
    pub person_id: i64,
    pub family_id: i64,
    pub relation_type: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct NewFamilyMember {
    pub person_id: i64,
    pub family_id: Option<i64>,
    pub new_family_last_name: Option<String>,
    pub relation_type: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct FamilyMemberDetail {
    pub id: i64,
    pub family_id: i64,
    pub relation_type: String,
    pub family_first_name: Option<String>,
    pub family_last_name: String,
    pub family_nickname: Option<String>,
}

/// Kombiniertes Paket für den Graph: alle Knoten + alle Kanten in einem Call,
/// damit das Frontend nicht zwei Requests synchronisieren muss.
#[derive(Debug, Serialize)]
pub struct GraphData {
    pub people: Vec<Person>,
    pub relationships: Vec<Relationship>,
}

#[derive(Debug, Deserialize)]
pub struct NominatimResult {
    pub lat: String,
    pub lon: String,
}
