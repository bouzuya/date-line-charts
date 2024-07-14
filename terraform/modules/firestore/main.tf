// <https://registry.terraform.io/providers/hashicorp/google/latest/docs/resources/firestore_database>
resource "google_firestore_database" "default" {
  name            = "(default)"
  location_id     = "nam5"
  type            = "FIRESTORE_NATIVE"
  deletion_policy = "DELETE"
}

// <https://registry.terraform.io/providers/hashicorp/google/latest/docs/resources/firestore_index>
resource "google_firestore_index" "index1" {
  collection = "col"
  database   = google_firestore_database.default.name

  fields {
    field_path = "name"
    order      = "ASCENDING"
  }
  fields {
    field_path = "description"
    order      = "DESCENDING"
  }
}
