fn allow() {
  if role == "admin" { return true; }
  if action == "profile:read" && subject_id == resource_owner { return true; }
  false
}
