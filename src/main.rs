mod card_operations;
mod utils;
mod dbo;

use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tokio::time::{self, Duration};
use crate::card_operations::card_operations::CardManager;
use crate::dbo::dbo::DboManager;
use crate::utils::utils::decimals_to_hex;

const TIME: Duration = Duration::from_secs(3);

/// La fonction `main` est asynchrone et utilise Tokio pour la gestion asynchrone des tâches. Elle crée une tâche asynchrone pour lire les cartes RFID périodiquement,
/// vérifier leur UUID dans la base de données, et gérer les entrées utilisateur via l'entrée standard.
///
/// Les commandes disponibles pour l'utilisateur sont :
/// - `add <nom_utilisateur>` : Ajoute un utilisateur avec le nom donné dans la base de données.
/// - `reset` : Supprime l'utilisateur associé à l'UUID de la carte lue de la base de données.
/// - `help` : Affiche les commandes disponibles.
/// - `exit` ou `quit` : Arrête le programme.
#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(1);

    tokio::spawn(async move {
        let mut interval = time::interval(TIME);
        loop {
            interval.tick().await;
            let card = CardManager{card: match CardManager::loadreader(){
            Ok(a) => a,
            Err(E) => {
                println!("{:?}", E);
                continue;
                }
            }};


            let carduuid = decimals_to_hex(card.read(0));

            let result: String = match DboManager::uuid_exist(&carduuid).await {
                Ok(uuid) => format!("Bienvenue {} !", uuid.to_owned()),
                Err(_) => String::from("Carte non configuré")
            };

            if let Err(_) = tx.send([result, carduuid]).await {
                break;
            }
        }
    });

    let stdin = io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();
    let mut last_message = [String::from("Message"), String::from("uuid")];

    loop {
        tokio::select! {
            Some(message) = rx.recv() => {
                if message != last_message {
                println!("{:?}", message);
                last_message = message; // Mettre à jour le dernier message affiché
                }
            }
            // Lire l'entrée utilisateur
            result = lines.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        let command = line.trim();
                        match command.split_whitespace().nth(0) {
                            Some("exit") | Some("quit") => {
                                println!("Arrêt de la boucle principale.");
                                break;
                            }
                            Some("add") => {
                                if let Some(arg) = command.split_whitespace().nth(1) {
                                    if let Some(msg) = rx.recv().await{
                                        DboManager::adduser(msg[1].clone(), &arg).await.expect("Erreur db");
                                    }
                                    println!("Ajout de l'utilisateur, {}!", arg);
                                } else {
                                    eprintln!("Merci de saisir au moins 1 nom d'utilisateur");
                                }
                            }
                            Some("reset") => {
                                if let Some(msg) = rx.recv().await{
                                        DboManager::deluser(msg[1].clone()).await.expect("Erreur db");
                                    }
                                    println!("Réinitialisation de la carte!");
                                }

                            Some("help") => {
                                println!("Commandes disponibles :");
                                println!("  add nomdutilisateur  - permet l'ajout d'une carte dans la base de donnée");
                                println!("  reset   - Supprime l'uuid de la carte dans la base de donnée");
                                println!("  exit   - Quitte le programme");
                            }
                            _ => {
                                println!("Commande inconnue: {}", command);
                                println!("Tapez 'help' pour voir la liste des commandes disponibles.");
                            }
                        }
                    }
                    Ok(None) => {
                        // Fin de l'entrée standard
                        break;
                    }
                    Err(e) => {
                        println!("Erreur de lecture de l'entrée: {}", e);
                        break;
                    }
                }
            }
        }

    }
}