use crate::models::{
    ContactEvent, FamilyMember, FamilyMemberDetail, GraphData, NewFamilyMember, NewPerson,
    NewRelationship, NominatimResult, Person, Relationship, UpdatePerson, UpdateRelationship,
};
use sqlx::SqlitePool;
use tauri::State;

type Db<'a> = State<'a, SqlitePool>;

const PERSON_COLUMNS: &str = "id, first_name, last_name, nickname, birth_date, known_since, address, employer, note, color, gender, image_path, image_crop_x, image_crop_y, image_crop_radius, lat, lon, geocoded_at, last_contact_type, last_contact_date, created_at";

fn attach_image(mut person: Person) -> Person {
    if let Some(path) = &person.image_path {
        person.image_data = crate::image::encode_data_url(std::path::Path::new(path));
    }
    person
}

fn attach_images(people: Vec<Person>) -> Vec<Person> {
    people.into_iter().map(attach_image).collect()
}

// ---------- People ----------

#[tauri::command]
pub async fn list_people(db: Db<'_>) -> Result<Vec<Person>, String> {
    let people = sqlx::query_as::<_, Person>(&format!(
        "SELECT {PERSON_COLUMNS} FROM people ORDER BY last_name, first_name"
    ))
    .fetch_all(db.inner())
    .await
    .map_err(|e| e.to_string())?;
    Ok(attach_images(people))
}

#[tauri::command]
pub async fn add_person(db: Db<'_>, payload: NewPerson) -> Result<Person, String> {
    let color = payload.color.unwrap_or_else(|| "#3b82f6".to_string());
    let rec = sqlx::query_as::<_, Person>(&format!(
        "INSERT INTO people (first_name, last_name, nickname, birth_date, known_since, address, employer, note, color, gender)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         RETURNING {PERSON_COLUMNS}"
    ))
    .bind(payload.first_name)
    .bind(payload.last_name)
    .bind(payload.nickname)
    .bind(payload.birth_date)
    .bind(payload.known_since)
    .bind(payload.address)
    .bind(payload.employer)
    .bind(payload.note)
    .bind(color)
    .bind(payload.gender)
    .fetch_one(db.inner())
    .await
    .map_err(|e| e.to_string())?;
    Ok(attach_image(rec))
}

#[tauri::command]
pub async fn update_person(db: Db<'_>, payload: UpdatePerson) -> Result<Person, String> {
    let rec = sqlx::query_as::<_, Person>(&format!(
        "UPDATE people SET first_name = ?, last_name = ?, nickname = ?, birth_date = ?, known_since = ?,
            lat = CASE WHEN address IS NOT ? THEN NULL ELSE lat END,
            lon = CASE WHEN address IS NOT ? THEN NULL ELSE lon END,
            geocoded_at = CASE WHEN address IS NOT ? THEN NULL ELSE geocoded_at END,
            address = ?, employer = ?, note = ?, color = ?, gender = ?
         WHERE id = ?
         RETURNING {PERSON_COLUMNS}"
    ))
    .bind(payload.first_name)
    .bind(payload.last_name)
    .bind(payload.nickname)
    .bind(payload.birth_date)
    .bind(payload.known_since)
    .bind(payload.address.clone())
    .bind(payload.address.clone())
    .bind(payload.address.clone())
    .bind(payload.address)
    .bind(payload.employer)
    .bind(payload.note)
    .bind(payload.color)
    .bind(payload.gender)
    .bind(payload.id)
    .fetch_one(db.inner())
    .await
    .map_err(|e| e.to_string())?;
    Ok(attach_image(rec))
}

#[tauri::command]
pub async fn delete_person(db: Db<'_>, id: i64) -> Result<(), String> {
    // Beziehungen werden durch ON DELETE CASCADE automatisch mitgelöscht.
    sqlx::query("DELETE FROM people WHERE id = ?")
        .bind(id)
        .execute(db.inner())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ---------- Profilbilder ----------

#[tauri::command]
pub async fn upload_person_image(
    db: Db<'_>,
    person_id: i64,
    file_bytes: Vec<u8>,
    crop_x: i64,
    crop_y: i64,
    crop_radius: i64,
) -> Result<Person, String> {
    let ext = crate::image::validate_image(&file_bytes)?;

    let dir = crate::image::images_dir();
    // Remove any previous image for this person, regardless of extension.
    let _ = std::fs::remove_file(dir.join(format!("{person_id}.jpg")));
    let _ = std::fs::remove_file(dir.join(format!("{person_id}.png")));

    let file_path = dir.join(format!("{person_id}.{ext}"));
    std::fs::write(&file_path, &file_bytes).map_err(|e| format!("Failed to save image: {e}"))?;

    let path_str = file_path.to_string_lossy().to_string();

    let rec = sqlx::query_as::<_, Person>(&format!(
        "UPDATE people SET image_path = ?, image_crop_x = ?, image_crop_y = ?, image_crop_radius = ?
         WHERE id = ?
         RETURNING {PERSON_COLUMNS}"
    ))
    .bind(&path_str)
    .bind(crop_x)
    .bind(crop_y)
    .bind(crop_radius)
    .bind(person_id)
    .fetch_one(db.inner())
    .await
    .map_err(|e| e.to_string())?;

    Ok(attach_image(rec))
}

#[tauri::command]
pub async fn delete_person_image(db: Db<'_>, person_id: i64) -> Result<Person, String> {
    let existing: Option<(Option<String>,)> =
        sqlx::query_as("SELECT image_path FROM people WHERE id = ?")
            .bind(person_id)
            .fetch_optional(db.inner())
            .await
            .map_err(|e| e.to_string())?;

    if let Some((Some(path),)) = existing {
        let _ = std::fs::remove_file(&path);
    }

    let rec = sqlx::query_as::<_, Person>(&format!(
        "UPDATE people SET image_path = NULL, image_crop_x = NULL, image_crop_y = NULL, image_crop_radius = NULL
         WHERE id = ?
         RETURNING {PERSON_COLUMNS}"
    ))
    .bind(person_id)
    .fetch_one(db.inner())
    .await
    .map_err(|e| e.to_string())?;

    Ok(attach_image(rec))
}

// ---------- Relationships ----------

fn check_distinct(a: i64, b: i64) -> Result<(), String> {
    if a == b {
        return Err("Eine Person kann nicht mit sich selbst verbunden werden".into());
    }
    Ok(())
}

/// Prüft, ob bereits eine Kante zwischen den beiden Personen existiert
/// (in beliebiger Richtung). SQLite kann UNIQUE auf einem ungeordneten
/// Paar nicht ausdrücken, darum wird das hier in Rust geprüft.
async fn pair_exists(db: &SqlitePool, a: i64, b: i64) -> Result<bool, String> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM relationships WHERE (from_id = ? AND to_id = ?) OR (from_id = ? AND to_id = ?)",
    )
    .bind(a)
    .bind(b)
    .bind(b)
    .bind(a)
    .fetch_one(db)
    .await
    .map_err(|e| e.to_string())?;
    Ok(count > 0)
}

#[tauri::command]
pub async fn list_relationships(db: Db<'_>) -> Result<Vec<Relationship>, String> {
    sqlx::query_as::<_, Relationship>(
        "SELECT id, from_id, to_id, kind, strength, note, created_at FROM relationships",
    )
    .fetch_all(db.inner())
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_relationship(
    db: Db<'_>,
    payload: NewRelationship,
) -> Result<Relationship, String> {
    check_distinct(payload.person_a, payload.person_b)?;
    if pair_exists(db.inner(), payload.person_a, payload.person_b).await? {
        return Err("Diese beiden Personen sind bereits verbunden".to_string());
    }

    let rec = sqlx::query_as::<_, Relationship>(
        "INSERT INTO relationships (from_id, to_id, kind, strength, note)
         VALUES (?, ?, ?, ?, ?)
         RETURNING id, from_id, to_id, kind, strength, note, created_at",
    )
    .bind(payload.person_a)
    .bind(payload.person_b)
    .bind(payload.kind)
    .bind(payload.strength)
    .bind(payload.note)
    .fetch_one(db.inner())
    .await
    .map_err(|e| e.to_string())?;
    Ok(rec)
}

#[tauri::command]
pub async fn swap_relationship_direction(db: Db<'_>, id: i64) -> Result<Relationship, String> {
    let rec = sqlx::query_as::<_, Relationship>(
        "UPDATE relationships SET from_id = to_id, to_id = from_id WHERE id = ?
         RETURNING id, from_id, to_id, kind, strength, note, created_at",
    )
    .bind(id)
    .fetch_one(db.inner())
    .await
    .map_err(|e| e.to_string())?;
    Ok(rec)
}

#[tauri::command]
pub async fn update_relationship(
    db: Db<'_>,
    payload: UpdateRelationship,
) -> Result<Relationship, String> {
    let rec = sqlx::query_as::<_, Relationship>(
        "UPDATE relationships SET kind = ?, strength = ?, note = ? WHERE id = ?
         RETURNING id, from_id, to_id, kind, strength, note, created_at",
    )
    .bind(payload.kind)
    .bind(payload.strength)
    .bind(payload.note)
    .bind(payload.id)
    .fetch_one(db.inner())
    .await
    .map_err(|e| e.to_string())?;
    Ok(rec)
}

#[tauri::command]
pub async fn delete_relationship(db: Db<'_>, id: i64) -> Result<(), String> {
    sqlx::query("DELETE FROM relationships WHERE id = ?")
        .bind(id)
        .execute(db.inner())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ---------- Familie ----------

#[tauri::command]
pub async fn get_family(db: Db<'_>, person_id: i64) -> Result<Vec<FamilyMemberDetail>, String> {
    sqlx::query_as::<_, FamilyMemberDetail>(
        "SELECT fm.id, fm.family_id, fm.relation_type,
                p.first_name AS family_first_name, p.last_name AS family_last_name, p.nickname AS family_nickname
         FROM family_members fm
         JOIN people p ON p.id = fm.family_id
         WHERE fm.person_id = ?",
    )
    .bind(person_id)
    .fetch_all(db.inner())
    .await
    .map_err(|e| e.to_string())
}

/// relation_type von person_id->family_id zu Sicht von family_id->person_id drehen.
/// other_gender ist das Geschlecht der Person, deren Rolle gerade benannt wird
/// (bei "Kind" der Elternteil, also family_id; sonst irrelevant).
fn inverse_relation_type(relation_type: &str, parent_gender: Option<&str>) -> String {
    match relation_type {
        "Kind" => match parent_gender {
            Some("w") => "Mutter".to_string(),
            Some("m") => "Vater".to_string(),
            _ => "Elternteil".to_string(),
        },
        "Mutter" | "Vater" => "Kind".to_string(),
        "Ehepartner" => "Ehepartner".to_string(),
        "Geschwister" => "Geschwister".to_string(),
        _ => "Sonstige".to_string(),
    }
}

#[tauri::command]
pub async fn add_family_member(
    db: Db<'_>,
    payload: NewFamilyMember,
) -> Result<FamilyMember, String> {
    let family_id = match payload.family_id {
        Some(id) => id,
        None => {
            let last_name = payload
                .new_family_last_name
                .ok_or("Name für neues Familienmitglied fehlt")?;
            let new_person = sqlx::query_as::<_, Person>(&format!(
                "INSERT INTO people (last_name) VALUES (?) RETURNING {PERSON_COLUMNS}"
            ))
            .bind(last_name)
            .fetch_one(db.inner())
            .await
            .map_err(|e| e.to_string())?;
            new_person.id
        }
    };

    let rec = sqlx::query_as::<_, FamilyMember>(
        "INSERT INTO family_members (person_id, family_id, relation_type) VALUES (?, ?, ?)
         RETURNING id, person_id, family_id, relation_type, created_at",
    )
    .bind(payload.person_id)
    .bind(family_id)
    .bind(&payload.relation_type)
    .fetch_one(db.inner())
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            "Diese Familienverbindung existiert bereits".to_string()
        } else {
            e.to_string()
        }
    })?;

    // Umgekehrten Eintrag anlegen, damit family_id ebenfalls "weiß", in welcher
    // Beziehung sie zu person_id steht (z.B. A ist Kind von B -> B ist Mutter von A).
    let parent_gender: Option<String> = sqlx::query_scalar("SELECT gender FROM people WHERE id = ?")
        .bind(family_id)
        .fetch_optional(db.inner())
        .await
        .ok()
        .flatten();
    let inverse_type = inverse_relation_type(&payload.relation_type, parent_gender.as_deref());

    let _ = sqlx::query(
        "INSERT INTO family_members (person_id, family_id, relation_type) VALUES (?, ?, ?)",
    )
    .bind(family_id)
    .bind(payload.person_id)
    .bind(&inverse_type)
    .execute(db.inner())
    .await;

    // person_id ist [relation_type] von family_id -> Richtung muss erhalten bleiben.
    if check_distinct(payload.person_id, family_id).is_ok()
        && !pair_exists(db.inner(), payload.person_id, family_id).await.unwrap_or(true)
    {
        let _ = sqlx::query(
            "INSERT INTO relationships (from_id, to_id, kind, strength) VALUES (?, ?, ?, 3)",
        )
        .bind(payload.person_id)
        .bind(family_id)
        .bind(&payload.relation_type)
        .execute(db.inner())
        .await;
    }

    Ok(rec)
}

#[tauri::command]
pub async fn remove_family_member(db: Db<'_>, person_id: i64, family_id: i64) -> Result<(), String> {
    sqlx::query("DELETE FROM family_members WHERE (person_id = ? AND family_id = ?) OR (person_id = ? AND family_id = ?)")
        .bind(person_id)
        .bind(family_id)
        .bind(family_id)
        .bind(person_id)
        .execute(db.inner())
        .await
        .map_err(|e| e.to_string())?;

    let _ = sqlx::query(
        "DELETE FROM relationships WHERE (from_id = ? AND to_id = ?) OR (from_id = ? AND to_id = ?)",
    )
    .bind(person_id)
    .bind(family_id)
    .bind(family_id)
    .bind(person_id)
    .execute(db.inner())
    .await;

    Ok(())
}

// ---------- Karte / Geocoding ----------

/// Geocodiert die Adresse einer Person über Nominatim (OpenStreetMap) und
/// speichert lat/lon. Nominatim verlangt einen aussagekräftigen User-Agent
/// und max. 1 Request/Sekunde – für eine Einzelperson pro Klick unkritisch.
#[tauri::command]
pub async fn geocode_person(db: Db<'_>, person_id: i64) -> Result<Person, String> {
    let row: Option<(Option<String>,)> = sqlx::query_as("SELECT address FROM people WHERE id = ?")
        .bind(person_id)
        .fetch_optional(db.inner())
        .await
        .map_err(|e| e.to_string())?;

    let address = match row {
        Some((Some(a),)) if !a.trim().is_empty() => a,
        _ => return Err("Keine Adresse hinterlegt".to_string()),
    };

    let client = reqwest::Client::new();
    let results: Vec<NominatimResult> = client
        .get("https://nominatim.openstreetmap.org/search")
        .query(&[("q", address.as_str()), ("format", "json"), ("limit", "1")])
        .header("User-Agent", "social-graph-desktop-app/0.1")
        .send()
        .await
        .map_err(|e| format!("Geocoding-Anfrage fehlgeschlagen: {e}"))?
        .json()
        .await
        .map_err(|e| format!("Geocoding-Antwort ungültig: {e}"))?;

    let hit = results
        .into_iter()
        .next()
        .ok_or("Adresse konnte nicht gefunden werden")?;

    let lat: f64 = hit.lat.parse().map_err(|_| "Ungültige Koordinate")?;
    let lon: f64 = hit.lon.parse().map_err(|_| "Ungültige Koordinate")?;

    let rec = sqlx::query_as::<_, Person>(&format!(
        "UPDATE people SET lat = ?, lon = ?, geocoded_at = datetime('now')
         WHERE id = ?
         RETURNING {PERSON_COLUMNS}"
    ))
    .bind(lat)
    .bind(lon)
    .bind(person_id)
    .fetch_one(db.inner())
    .await
    .map_err(|e| e.to_string())?;

    Ok(attach_image(rec))
}

// ---------- Kontakt-Verlauf ----------

/// Berechnet den denormalisierten "letzter Kontakt"-Cache auf `people` neu
/// (= Event mit höchstem Datum) und gibt die aktualisierte Person zurück.
async fn refresh_last_contact(db: &SqlitePool, person_id: i64) -> Result<Person, String> {
    let rec = sqlx::query_as::<_, Person>(&format!(
        "UPDATE people SET
            last_contact_type = (SELECT contact_type FROM contact_events
                                 WHERE person_id = ? ORDER BY contact_date DESC, id DESC LIMIT 1),
            last_contact_date = (SELECT contact_date FROM contact_events
                                 WHERE person_id = ? ORDER BY contact_date DESC, id DESC LIMIT 1)
         WHERE id = ?
         RETURNING {PERSON_COLUMNS}"
    ))
    .bind(person_id)
    .bind(person_id)
    .bind(person_id)
    .fetch_one(db)
    .await
    .map_err(|e| e.to_string())?;
    Ok(attach_image(rec))
}

#[tauri::command]
pub async fn list_contact_events(
    db: Db<'_>,
    person_id: i64,
) -> Result<Vec<ContactEvent>, String> {
    sqlx::query_as::<_, ContactEvent>(
        "SELECT id, person_id, contact_type, contact_date, note, created_at
         FROM contact_events WHERE person_id = ?
         ORDER BY contact_date DESC, id DESC",
    )
    .bind(person_id)
    .fetch_all(db.inner())
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_contact_event(
    db: Db<'_>,
    person_id: i64,
    contact_type: String,
    contact_date: String,
    note: Option<String>,
) -> Result<Person, String> {
    sqlx::query(
        "INSERT INTO contact_events (person_id, contact_type, contact_date, note)
         VALUES (?, ?, ?, ?)",
    )
    .bind(person_id)
    .bind(contact_type)
    .bind(contact_date)
    .bind(note)
    .execute(db.inner())
    .await
    .map_err(|e| e.to_string())?;

    refresh_last_contact(db.inner(), person_id).await
}

#[tauri::command]
pub async fn delete_contact_event(db: Db<'_>, event_id: i64) -> Result<Person, String> {
    let person_id: i64 = sqlx::query_scalar(
        "SELECT person_id FROM contact_events WHERE id = ?",
    )
    .bind(event_id)
    .fetch_one(db.inner())
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM contact_events WHERE id = ?")
        .bind(event_id)
        .execute(db.inner())
        .await
        .map_err(|e| e.to_string())?;

    refresh_last_contact(db.inner(), person_id).await
}

// ---------- Graph (kombiniert) ----------

/// Liefert Knoten und Kanten in einem einzigen Call, damit das Frontend
/// den Graphen ohne zwei separate Requests aufbauen muss.
#[tauri::command]
pub async fn get_graph(db: Db<'_>) -> Result<GraphData, String> {
    let people = sqlx::query_as::<_, Person>(&format!(
        "SELECT {PERSON_COLUMNS} FROM people ORDER BY last_name, first_name"
    ))
    .fetch_all(db.inner())
    .await
    .map_err(|e| e.to_string())?;

    let relationships = sqlx::query_as::<_, Relationship>(
        "SELECT id, from_id, to_id, kind, strength, note, created_at FROM relationships",
    )
    .fetch_all(db.inner())
    .await
    .map_err(|e| e.to_string())?;

    Ok(GraphData {
        people: attach_images(people),
        relationships,
    })
}
