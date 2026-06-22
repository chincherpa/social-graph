import { invoke } from "@tauri-apps/api/core";

export function displayName(person) {
  if (!person) return "?";
  if (person.nickname) return person.nickname;
  return [person.first_name, person.last_name].filter(Boolean).join(" ");
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

// ---------- Graph ----------

export function getGraph() {
  return invoke("get_graph");
}
