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
  "Mutter",
  "Sohn",
  "Tante",
  "Tochter",
  "Vater",
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
  Mutter: { m: "Sohn", w: "Tochter", default: "Kind" },
  Vater: { m: "Sohn", w: "Tochter", default: "Kind" },
};

// Bezeichnung für to_id (das Gegenstück eines gerichteten kind), abhängig von dessen Geschlecht.
// Beispiel: kind="Tochter", toGender="w" -> "Mutter".
export function reciprocalKind(kind, toGender) {
  const entry = reciprocalKinds[kind];
  if (!entry) return kind;
  if (typeof entry === "string") return entry;
  return entry[toGender] ?? entry.default;
}

// ---------- Abgeleitete Beziehungen ----------

// kinds, bei denen from_id das Kind von to_id ist.
const childOfKinds = new Set(["Sohn", "Tochter"]);
// kinds, bei denen from_id der Elternteil von to_id ist.
const parentOfKinds = new Set(["Mutter", "Vater"]);
// kinds, die eine Geschwisterbeziehung bereits explizit ausdrücken.
export const siblingKinds = new Set(["Bruder", "Schwester", "Geschwister"]);

// Alle Eltern-Kind-Kanten als {parent, child} normalisiert.
function parentChildLinks(relationships) {
  const links = [];
  for (const r of relationships) {
    if (childOfKinds.has(r.kind)) links.push({ parent: r.to_id, child: r.from_id });
    else if (parentOfKinds.has(r.kind)) links.push({ parent: r.from_id, child: r.to_id });
  }
  return links;
}

// Geschwister-IDs einer Person, abgeleitet über gemeinsame Eltern.
// Beispiel: A Tochter von C, B Sohn von C -> A und B sind Geschwister.
export function deriveSiblingIds(personId, relationships) {
  const links = parentChildLinks(relationships);
  const parents = new Set(links.filter((l) => l.child === personId).map((l) => l.parent));
  const siblingIds = new Set();
  for (const l of links) {
    if (l.child !== personId && parents.has(l.parent)) siblingIds.add(l.child);
  }
  return siblingIds;
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

// ---------- Letzter Kontakt ----------

export const contactTypes = [
  { value: "in_person", label: "in person" },
  { value: "messenger", label: "via messenger" },
  { value: "call", label: "call" },
  { value: "email", label: "email" },
];

export function contactTypeLabel(value) {
  return contactTypes.find((t) => t.value === value)?.label ?? value;
}

const contactIcons = {
  in_person: "🤝",
  messenger: "💬",
  call: "📞",
  email: "✉️",
};

export function contactTypeIcon(value) {
  return contactIcons[value] ?? "•";
}

export function listContactEvents(personId) {
  return invoke("list_contact_events", { personId });
}

export function addContactEvent(personId, contactType, contactDate, note) {
  return invoke("add_contact_event", {
    personId,
    contactType,
    contactDate,
    note: note || null,
  });
}

export function deleteContactEvent(eventId) {
  return invoke("delete_contact_event", { eventId });
}

// ---------- Graph ----------

export function getGraph() {
  return invoke("get_graph");
}
