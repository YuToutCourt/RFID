//! # Carte bancaire simulation en Rust
//!
//! Ce programme simule les opérations bancaires de base (dépôt et retrait) en utilisant une carte à puce.
//! Il utilise la bibliothèque `pcsc` pour interagir avec le lecteur de carte à puce et la carte.
//!
//! ## Fonctionnalités
//! - Charger une clé de carte
//! - Authentifier la carte
//! - Lire et écrire des blocs de données sur la carte
//! - Convertir des valeurs hexadécimales en tableaux d'octets et vice versa
//! - Convertir des valeurs hexadécimales en décimales

mod card_operations;
mod utils;

use card_operations::card_operations::CardManager;
use utils::utils::{hexa_to_decimal, hexa_to_tableau};

use pcsc::*;
use std::io::{self, Write};


/// Point d'entrée principal du programme.
///
/// # Explication
///
/// Cette fonction représente le point d'entrée principal du programme. Elle initialise le contexte PC/SC, récupère la liste des lecteurs de carte disponibles, sélectionne le premier lecteur disponible, puis se connecte à la carte. Ensuite, elle initialise la balance en convertissant les données du bloc 4 de la carte en décimal.
///
/// Ensuite, la fonction entre dans une boucle principale pour interagir avec la carte et effectuer des opérations de dépôt et de retrait. À chaque itération, elle affiche la balance actuelle, attend une entrée de l'utilisateur pour une commande (dépôt ou retrait), puis traite la commande entrée. Si la commande est valide, elle effectue l'opération correspondante (dépôt ou retrait) et met à jour la balance sur la carte.
///
/// # Arguments
///
/// Aucun.
///
/// # Exemple
///
/// ```rust
/// // Le programme est exécuté en tant que point d'entrée principal
/// fn main() {
///     // Code principal du programme
/// }
/// ```
fn main() {
    // Initialise le contexte PC/SC
    let ctx = Context::establish(Scope::User).expect("Failed to establish context");

    // Initialise un tampon pour les lecteurs de carte
    let mut readers_buf = [0; 2048];

    // Récupère la liste des lecteurs de carte disponibles
    let mut readers = match ctx.list_readers(&mut readers_buf) {
        Ok(readers) => readers,
        Err(err) => {
            eprintln!("Failed to list readers: {}", err);
            std::process::exit(1);
        }
    };

    // Sélectionne le premier lecteur disponible
    let reader = match readers.next() {
        Some(reader) => reader,
        None => {
            println!("No readers are connected.");
            return;
        }
    };

    // Connecte à la carte avec le lecteur sélectionné
    let card = ctx.connect(reader, ShareMode::Shared, Protocols::ANY)
        .expect("Failed to connect to the card");

    println!("Connected to the card");

    // Initialise la balance en convertissant les données du bloc 4 de la carte en décimal
    let mut balance = hexa_to_decimal(read(&card, 4));

    // Boucle principale pour interagir avec la carte et effectuer des opérations de dépôt et de retrait
    loop {
        println!("Balance actuelle: {}", balance);
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");

        let input = input.trim();
        let parts: Vec<&str> = input.split_whitespace().collect();

        // Vérifie la validité de la commande entrée
        if parts.len() != 2 {
            println!("Commande invalide. Utilisation: 'dépôt montant' ou 'retrait montant'");
            continue;
        }

        let command = parts[0];
        let amount: Result<i128, _> = parts[1].parse();

        // Vérifie si le montant est valide
        if let Ok(amount) = amount {
            match command {
                "depot" => {
                    // Effectue un dépôt
                    balance += amount;
                    let hexa = format!("{:0>32X}", balance);
                    write(&card, 4, hexa_to_tableau(hexa));
                    println!("Dépôt de {} effectué. Nouvelle balance: {}", amount, balance);
                }
                "retrait" => {
                    // Effectue un retrait
                    if amount <= balance {
                        balance -= amount;
                        let hexa = format!("{:0>32X}", balance);
                        write(&card, 4, hexa_to_tableau(hexa));
                        println!("Retrait de {} effectué. Nouvelle balance: {}", amount, balance);
                    } else {
                        println!("Solde insuffisant pour effectuer le retrait.");
                    }
                }
                _ => {
                    println!("Commande invalide. Utilisation: 'dépôt montant' ou 'retrait montant'");
                }
            }
        } else {
            println!("Montant invalide.");
        }
    }
}

