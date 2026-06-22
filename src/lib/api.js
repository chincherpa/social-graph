import { invoke } from "@tauri-apps/api/core";

export function displayName(person) {
  if (!person) return "?";
  if (person.nickname) return person.nickname;
  return [person.first_name, person.last_name].filter(Boolean).join(" ");
}

// kinds, bei denen from_id -> to_id eine Richtung bedeutet
// (z.B. "Tochter": from_id ist die Tochter von to_id).
export const directionalKinds = new Set([
  "Bruder",
  "Ehefrau",
  "Ehemann",
  "Enkel",
  "Enkelin",
  "Ex-Ehefrau",
  "Ex-Ehemann",
  "Oma",
  "Onkel",
  "Opa",
  "Schwester",
  "Sohn",
  "Tante",
  "Tochter",
]);

// Für gerichtete kinds: was ist to_id für from_id, ausgedrückt aus Sicht von to_id
// (dessen Geschlecht die Bezeichnung bestimmt, z.B. Tochter -> Mutter/Vater).
const reciprocalKinds = {
  Bruder: { m: "Bruder", w: "Schwester", default: "Geschwister" },
  Schwester: { m: "Bruder", w: "Schwester", default: "Geschwister" },
  Ehefrau: "Ehemann",
  Ehemann: "Ehefrau",
  "Ex-Ehefrau": "Ex-Ehemann",
  "Ex-Ehemann": "Ex-Ehefrau",
  Enkel: { m: "Opa", w: "Oma", default: "Großelternteil" },
  Enkelin: { m: "Opa", w: "Oma", default: "Großelternteil" },
  Oma: { m: "Enkel", w: "Enkelin", default: "Enkelkind" },
  Opa: { m: "Enkel", w: "Enkelin", default: "Enkelkind" },
  Onkel: { m: "Neffe", w: "Nichte", default: "Neffe/Nichte" },
  Tante: { m: "Neffe", w: "Nichte", default: "Neffe/Nichte" },
  Sohn: { m: "Vater", w: "Mutter", default: "Elternteil" },
  Tochter: { m: "Vater", w: "Mutter", default: "Elternteil" },
};

// Bezeichnung für to_id (das Gegenstück eines gerichteten kind), abhängig von dessen Geschlecht.
// Beispiel: kind="Tochter", toGender="w" -> "Mutter".
export function reciprocalKind(kind, toGender) {
  const entry = reciprocalKinds[kind];
  if (!entry) return kind;
  if (typeof entry === "string") return entry;
  return entry[toGender] ?? entry.default;
}

// ---------- People ----------

export function listPeople() {
  return invoke("list_people");
}

export function addPerson({
  firstName,
  lastName,
  nickname,
  birthDate,
  knownSince,
  address,
  employer,
  note,
  color,
  gender,
}) {
  return invoke("add_person", {
    payload: {
      first_name: firstName,
      last_name: lastName,
      nickname,
      birth_date: birthDate,
      known_since: knownSince,
      address,
      employer,
      note,
      color,
      gender,
    },
  });
}

export function updatePerson({
  id,
  firstName,
  lastName,
  nickname,
  birthDate,
  knownSince,
  address,
  employer,
  note,
  color,
  gender,
}) {
  return invoke("update_person", {
    payload: {
      id,
      first_name: firstName,
      last_name: lastName,
      nickname,
      birth_date: birthDate,
      known_since: knownSince,
      address,
      employer,
      note,
      color,
      gender,
    },
  });
}

export function deletePerson(id) {
  return invoke("delete_person", { id });
}

// ---------- Profilbilder ----------

export function placeholderFor(gender) {
  return gender === "w" ? "images/female.png" : "images/male.png";
}

export async function uploadPersonImage(personId, fileBytes, cropX, cropY, cropRadius) {
  return invoke("upload_person_image", {
    personId,
    fileBytes: Array.from(fileBytes),
    cropX,
    cropY,
    cropRadius,
  });
}

export function deletePersonImage(personId) {
  return invoke("delete_person_image", { personId });
}

// ---------- Relationships ----------

export function listRelationships() {
  return invoke("list_relationships");
}

export function addRelationship({ personA, personB, kind, strength, note }) {
  return invoke("add_relationship", {
    payload: { person_a: personA, person_b: personB, kind, strength, note },
  });
}

export function updateRelationship({ id, kind, strength, note }) {
  return invoke("update_relationship", { payload: { id, kind, strength, note } });
}

export function deleteRelationship(id) {
  return invoke("delete_relationship", { id });
}

export function swapRelationshipDirection(id) {
  return invoke("swap_relationship_direction", { id });
}

// ---------- Familie ----------

export function getFamily(personId) {
  return invoke("get_family", { personId });
}

export function addFamilyMember({ personId, familyId, newFamilyLastName, relationType }) {
  return invoke("add_family_member", {
    payload: {
      person_id: personId,
      family_id: familyId ?? null,
      new_family_last_name: newFamilyLastName ?? null,
      relation_type: relationType,
    },
  });
}

export function removeFamilyMember(personId, familyId) {
  return invoke("remove_family_member", { personId, familyId });
}

// ---------- Karte ----------

export function geocodePerson(personId) {
  return invoke("geocode_person", { personId });
}

// ---------- Graph ----------

export function getGraph() {
  return invoke("get_graph");
}
