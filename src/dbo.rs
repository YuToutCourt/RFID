
/// Module `dbo` fournit des fonctions pour gérer les opérations de base de données
/// en utilisant SQLx avec SQLite.
pub mod dbo {
    const DB_URL: &str = "sqlite://sqlite3.db";
    use sqlx::{Error, Pool, Row, Sqlite, SqlitePool};
    use serde_json::json;
    use std::fs::File;
    use std::io::Write;

    /// Structure `DboManager` gère les opérations sur la base de données.

    pub struct DboManager {
        pub dboconnector: SqlitePool,
    }


    /// Établit une connexion à la base de données.
    ///
    /// # Retourne
    ///
    /// * `Pool<Sqlite>` - La connexion à la base de données.
    ///
    /// # Exemples
    ///
    /// ```
    /// let db = DboManager::dbconnection().await;
    /// ```

    impl DboManager {
        async fn dbconnection() -> Pool<Sqlite> {
            let connection = SqlitePool::connect(DB_URL).await.unwrap();
            connection
        }

        /// Vérifie si un UUID existe dans la table des utilisateurs.
        ///
        /// # Arguments
        ///
        /// * `uuid` - Une référence à une chaîne représentant l'UUID à vérifier.
        ///
        /// # Retourne
        ///
        /// * `Result<String, Error>` - Le nom de l'utilisateur associé à l'UUID s'il existe, sinon une erreur.
        ///
        /// # Exemples
        ///
        /// ```
        /// let uuid_exists = DboManager::uuid_exist("some-uuid").await;
        /// ```

        pub async fn uuid_exist(uuid: &str) -> Result<String, Error> {
            let db = Self::dbconnection().await;
            let query = format!("SELECT * FROM users where uuid = '{}' LIMIT 1", uuid);
            match sqlx::query(&query).fetch_optional(&db).await {
                Ok(Some(row)) => {
                    let uuid: String = row.get("name");
                    db.close().await;
                    Ok(uuid)
                },
                Ok(None) => Err(Error::RowNotFound),
                Err(e) => Err(e),
            }
        }

        /// Ajoute un utilisateur à la table des utilisateurs.
        ///
        /// # Arguments
        ///
        /// * `uuid` - Une chaîne représentant l'UUID de l'utilisateur.
        /// * `username` - Une référence à une chaîne représentant le nom de l'utilisateur.
        ///
        /// # Retourne
        ///
        /// * `Result<u64, Error>` - Le nombre de lignes affectées par l'insertion.
        ///
        /// # Exemples
        ///
        /// ```
        /// let rows_affected = DboManager::adduser("some-uuid".to_string(), "username").await;
        /// ```

        pub async fn adduser(uuid: String, username: &str) -> Result<u64, Error> {
            let db = Self::dbconnection().await;
            let query = "INSERT INTO users (uuid, name) VALUES (?, ?)".to_string();
            let result = sqlx::query(&query).bind(uuid).bind(username).execute(&db).await?;
            Ok(result.rows_affected())
        }

        /// Supprime un utilisateur de la table des utilisateurs.
        ///
        /// # Arguments
        ///
        /// * `uuid` - Une chaîne représentant l'UUID de l'utilisateur à supprimer.
        ///
        /// # Retourne
        ///
        /// * `Result<u64, Error>` - Le nombre de lignes affectées par la suppression.
        ///
        /// # Exemples
        ///
        /// ```
        /// let rows_affected = DboManager::deluser("some-uuid".to_string()).await;
        /// ```
        pub async fn deluser(uuid: String) -> Result<u64, Error> {
            let db = Self::dbconnection().await;
            let query = "DELETE FROM users WHERE uuid = ?".to_string();
            let result = sqlx::query(&query).bind(uuid).execute(&db).await?;
            Ok(result.rows_affected())
        }

        /// Exporte les utilisateurs au format JSON.
        ///
        /// # Retourne
        ///
        /// * `Result<String, Error>` - Les utilisateurs sous forme de chaîne JSON en cas de succès, sinon une erreur.
        ///
        /// # Exemples
        ///
        /// ```
        /// let json_data = DboManager::export_users_to_json(file_path).await;
        /// ```

        pub async fn export_users_to_json(file_path: &str) -> Result<(), Error>  {
            let db = Self::dbconnection().await;
            let query = "SELECT * FROM users";
            let rows = sqlx::query(query).fetch_all(&db).await?;

            let users: Vec<_> = rows.iter().map(|row| {
                let uuid: String = row.get("uuid");
                let name: String = row.get("name");
                json!({"uuid": uuid, "name": name})
            }).collect();

            let json_data = json!(users).to_string();

            db.close().await;

            let mut file = File::create(file_path)?;
            file.write_all(json_data.as_bytes())?;

            Ok(())
        }
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        use std::fs;


        #[tokio::test]
        async fn test_dbconnection() {
            let result = super::DboManager::dbconnection().await;
            assert!(!result.is_closed());
        }

        #[tokio::test]
        async fn test_uuid_exist_existing_uuid() {
            let existing_uuid = "A4504FA11A8406263646566676869";
            let expected_name = "tonton";
            let result = DboManager::uuid_exist(existing_uuid).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), expected_name);
        }



        async fn test_adduser() {
            let _uuid = "FFFFFFFFFFFFFFFFFFFFFFFFFFFF";
            let _name = "MIKU";

            let result = DboManager::adduser(_uuid.parse().unwrap(), _name).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 1);
        }


        async fn test_deluser() {
            let _uuid = "FFFFFFFFFFFFFFFFFFFFFFFFFFFF";

            let result = DboManager::deluser(_uuid.parse().unwrap()).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 1);
        }

        #[tokio::test]
        async fn test_add_and_del_user() {
            test_adduser().await;
            test_deluser().await;
        }


        #[tokio::test]
        async fn test_export_users_to_json_file() {
            let file_path = "export_test_users.json";
            let result = DboManager::export_users_to_json(file_path).await;
            assert!(result.is_ok());

            let json_data = fs::read_to_string(file_path).unwrap();
            println!("{}", json_data);

        }


    }

}