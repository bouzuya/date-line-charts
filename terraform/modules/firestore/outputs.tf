output "database_name" {
  description = "The name of the created database"
  value       = google_firestore_database.default.id
}

output "index1_name" {
  description = "The name of the created index1"
  value       = google_firestore_index.index1.name
}
