use crate::models::{
    FamilyMember, FamilyMemberDetail, GraphData, NewFamilyMember, NewPerson, NewRelationship,
    Person, Relationship, UpdatePerson, UpdateRelationship,
};
use sqlx::SqlitePool;
use tauri::State;

type Db<'a> = State<'a, SqlitePool>;

const PERSON_COLUMNS: &str = "id, first_name, last_name, nickname, birth_date, known_since, address, employer, note, color, gender, created_at";

// ---------- People ----------

#[tauri::command]
pub async fn list_people(db: Db<'_>) -> Result<Vec<Person>, String> {
    sqlx::query_as::<_, Person>(&format!(
        "SELECT {PERSON_COLUMNS} FROM people ORDER BY last_name, first_name"
    ))
    .fetch_all(db.inner())
    .await
    .map_err(|e| e.to_string())
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
    Ok(rec)
}

#[tauri::command]
pub async fn update_person(db: Db<'_>, payload: UpdatePerson) -> Result<Person, String> {
    let rec = sqlx::query_as::<_, Person>(&format!(
        "UPDATE people SET first_name = ?, last_name = ?, nickname = ?, birth_date = ?, known_since = ?, address = ?, employer = ?, note = ?, color = ?, gender = ?
         WHERE id = ?
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
    .bind(payload.color)
    .bind(payload.gender)
    .bind(payload.id)
    .fetch_one(db.inner())
    .await
    .map_err(|e| e.to_string())?;
    Ok(rec)
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

// ---------- Relationships ----------

/// Normalisiert ein Personen-Paar so, dass from_id < to_id gilt.
/// Das hält die UNIQUE-Constraint konsistent, egal in welcher
/// Reihenfolge der Nutzer im Frontend zwei Knoten verbindet.
fn normalize_pair(a: i64, b: i64) -> Result<(i64, i64), String> {
    if a == b {
        return Err("Eine Person kann nicht mit sich selbst verbunden werden".into());
    }
    Ok(if a < b { (a, b) } else { (b, a) })
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
    let (from_id, to_id) = normalize_pair(payload.person_a, payload.person_b)?;

    let rec = sqlx::query_as::<_, Relationship>(
        "INSERT INTO relationships (from_id, to_id, kind, strength, note)
         VALUES (?, ?, ?, ?, ?)
         RETURNING id, from_id, to_id, kind, strength, note, created_at",
    )
    .bind(from_id)
    .bind(to_id)
    .bind(payload.kind)
    .bind(payload.strength)
    .bind(payload.note)
    .fetch_one(db.inner())
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            "Diese beiden Personen sind bereits verbunden".to_string()
        } else {
            e.to_string()
        }
    })?;
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

    if let Ok((from_id, to_id)) = normalize_pair(payload.person_id, family_id) {
        let _ = sqlx::query(
            "INSERT INTO relationships (from_id, to_id, kind, strength) VALUES (?, ?, ?, 3)",
        )
        .bind(from_id)
        .bind(to_id)
        .bind(&payload.relation_type)
        .execute(db.inner())
        .await;
    }

    Ok(rec)
}

#[tauri::command]
pub async fn remove_family_member(db: Db<'_>, person_id: i64, family_id: i64) -> Result<(), String> {
    sqlx::query("DELETE FROM family_members WHERE person_id = ? AND family_id = ?")
        .bind(person_id)
        .bind(family_id)
        .execute(db.inner())
        .await
        .map_err(|e| e.to_string())?;

    if let Ok((from_id, to_id)) = normalize_pair(person_id, family_id) {
        let _ = sqlx::query("DELETE FROM relationships WHERE from_id = ? AND to_id = ?")
            .bind(from_id)
            .bind(to_id)
            .execute(db.inner())
            .await;
    }

    Ok(())
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
        people,
        relationships,
    })
}
