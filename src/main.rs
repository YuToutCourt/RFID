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

use pcsc::*;
use std::io::{self, Write};

/// Charge la clé de la carte.
///
/// # Arguments
///
/// * `card` - Référence à l'objet `Card` représentant la connexion à la carte.
/// * `key` - Tableau de 6 octets représentant la clé à charger.
///
/// # Explication
///
/// Cette fonction charge une clé d'authentification dans le lecteur de carte à puce. Le processus est le suivant :
///
/// 1. **Préparation de l'APDU de chargement de clé** :
///    Un APDU (Application Protocol Data Unit) de commande est préparé avec les champs suivants :
///    - `0xFF` : Classe de commande.
///    - `0x82` : Instruction pour charger une clé d'authentification.
///    - `0x00` : P1 (structure de clé).
///    - `0x00` : P2 (emplacement de clé).
///    - `0x06` : Lc (longueur du champ de données).
///    - `key[0]..key[5]` : Les 6 octets de la clé à charger.
///
/// 2. **Transmission de l'APDU et réception des données** :
///    La commande APDU préparée est transmise à la carte à puce en utilisant la méthode `transmit`.
///    La réponse de la carte est stockée dans le tableau `rapdu`.
///
/// 3. **Vérification du statut de l'opération** :
///    La fonction vérifie le code de statut de la réponse APDU. Les deux premiers octets de la réponse (`rapdu[..2]`) sont comparés à `[0x90, 0x00]`, indiquant une opération réussie.
///    - Si le code de statut n'est pas `[0x90, 0x00]`, un message d'erreur est affiché indiquant l'échec du chargement de la clé.
///
/// # Notes
///
/// - `MAX_BUFFER_SIZE` doit être défini quelque part dans votre code pour représenter la taille maximale du tampon pour la réponse APDU.
/// - La structure et l'emplacement de la clé peuvent varier en fonction des spécifications de la carte et du lecteur.
///
/// # Exemple
///
/// ```
/// let card: Card = ...; // Code pour initialiser et connecter la carte
/// keyload(&card, [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]); // Charge une clé par défaut
/// ```
///
fn keyload(card: &Card, key: [u8; 6]) {
    let load_key_apdu = [
        0xFF, // Class
        0x82, // INS: Load Authentication Key
        0x00, // P1: Key Structure
        0x00, // P2: Key Slot (0)
        0x06, // Lc: Length of Key
        key[0], key[1], key[2], key[3], key[4], key[5] // Key A
    ];

    let mut rapdu = [0; 256];
    card.transmit(&load_key_apdu, &mut rapdu).expect("Impossible de charger les clés");

    let status_word = &rapdu[..2];
    if status_word != [0x90, 0x00] {
        eprintln!("Load key failed with status: {:02X?}", status_word);
        return;
    }
}


/// Authentifie un bloc spécifique de la carte.
///
/// # Arguments
///
/// * `card` - Référence à l'objet `Card` représentant la connexion à la carte.
/// * `block` - Numéro du bloc à authentifier.
///
/// # Explication
///
/// Cette fonction authentifie un bloc spécifique sur une carte à puce, permettant ainsi l'accès aux données de ce bloc. Le processus est le suivant :
///
/// 1. **Préparation de l'APDU d'authentification** :
///    Un APDU (Application Protocol Data Unit) de commande est préparé avec les champs suivants :
///    - `0xFF` : Classe de commande.
///    - `0x86` : Instruction pour l'authentification générale.
///    - `0x00` : P1 (paramètre 1).
///    - `0x00` : P2 (paramètre 2).
///    - `0x05` : Lc (longueur du champ de données).
///    - `0x01` : Numéro de version.
///    - `0x00` : Inconnu, peut-être un byte de réserve.
///    - `block` : Numéro du bloc à authentifier.
///    - `0x60` : Type de clé (0x60 pour la clé A).
///    - `0x00` : Numéro de la clé (0 pour la clé chargée).
///
/// 2. **Transmission de l'APDU et réception des données** :
///    La commande APDU préparée est transmise à la carte à puce en utilisant la méthode `transmit`.
///    La réponse de la carte est stockée dans le tableau `rapdu`.
///
/// 3. **Vérification du statut de l'opération** :
///    La fonction vérifie le code de statut de la réponse APDU. Le premier octet de la réponse (`rapdu[0]`) est comparé à `0x90`, qui indique une opération réussie.
///    - Si le code de statut n'est pas `0x90`, un message d'erreur est affiché indiquant l'échec de l'authentification.
///
/// # Notes
///
/// - `MAX_BUFFER_SIZE` doit être défini quelque part dans votre code pour représenter la taille maximale du tampon pour la réponse APDU.
/// - Le code vérifie seulement le premier octet de la réponse pour le code de statut, mais généralement les deux derniers octets sont utilisés pour indiquer le statut dans les réponses APDU.
///   Cela pourrait être une erreur potentielle dans la vérification du statut, à moins que cela ne soit conforme aux spécifications de votre carte particulière.
///
/// # Exemple
///
/// ```
/// let card: Card = ...; // Code pour initialiser et connecter la carte
/// auth(&card, 4); // Authentifie le bloc 4
/// ```
///
fn auth(card: &Card, block: u8) {
    let auth_apdu = [
        0xFF, // Class
        0x86, // INS: General Authenticate
        0x00, // P1
        0x00, // P2
        0x05, // Lc: Length of data field
        0x01, // Version number
        0x00,
        block, // Block number (block 0 for sector 0)
        0x60, // Key type (A)
        0x00, // Key number (0 for loaded key)
    ];
    let mut rapdu = [0; MAX_BUFFER_SIZE];
    card.transmit(&auth_apdu, &mut rapdu).expect("Failed to authenticate");

    let status_word = rapdu[0];
    if status_word != 0x90 {
        eprintln!("Auth failed: {:02X?}", status_word);
        return;
    }
}


/// Lit les données d'un bloc spécifique de la carte.
///
/// # Arguments
///
/// * `card` - Référence à l'objet `Card` représentant la connexion à la carte.
/// * `block` - Numéro du bloc à lire.
///
/// # Retourne
///
/// Un vecteur d'octets contenant les données lues.
///
/// # Explication
///
/// Cette fonction lit les données d'un bloc spécifique de la carte à puce. Le processus est le suivant :
///
/// 1. **Chargement de la clé d'authentification** :
///    La fonction `keyload` est appelée avec une clé par défaut (tous les octets à 0xFF) pour charger la clé d'authentification dans le lecteur.
///    Un message "Load key successful" est imprimé si le chargement est réussi.
///
/// 2. **Authentification du bloc** :
///    La fonction `auth` est appelée pour authentifier l'accès au bloc spécifié à l'aide de la clé précédemment chargée.
///
/// 3. **Préparation de l'APDU de lecture** :
///    Un APDU (Application Protocol Data Unit) de commande est préparé avec les champs suivants :
///    - `0xFF` : Classe de commande.
///    - `0xB0` : Instruction pour lire des données binaires.
///    - `0x00` : P1 (offset supérieur).
///    - `block` : P2 (offset inférieur, le numéro du bloc).
///    - `0x10` : Nombre d'octets à lire (16 octets pour un bloc).
///
/// 4. **Transmission de l'APDU et réception des données** :
///    La commande APDU préparée est transmise à la carte à puce en utilisant la méthode `transmit`.
///    La réponse de la carte est stockée dans le tableau `rapdu`.
///
/// 5. **Vérification du statut de l'opération et extraction des données** :
///    La fonction vérifie le code de statut de la réponse APDU. Elle extrait les 16 premiers octets des données lues et les retourne.
///    - Si `rapdu[16]` (le 17ème octet) n'est pas égal à `0x90` (indiquant une opération réussie), elle retourne les 16 premiers octets de `rapdu`.
///    - Sinon, elle retourne également les 16 premiers octets de `rapdu`.
///
/// # Notes
///
/// - `MAX_BUFFER_SIZE` doit être défini quelque part dans votre code pour représenter la taille maximale du tampon pour la réponse APDU.
/// - Le code vérifie l'octet 16 de la réponse APDU, mais souvent le code de statut est dans les deux derniers octets de la réponse.
///   Cela pourrait être une erreur potentielle dans la vérification du statut, à moins que cela ne soit conforme aux spécifications de votre carte particulière.
///
/// # Exemple
///
/// ```
/// let card: Card = ...; // Code pour initialiser et connecter la carte
/// let block_data = read(&card, 4);
/// println!("{:?}", block_data);
/// ```
///
fn read(card: &Card, block: u8) -> Vec<u8> {
    keyload(&card, [0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);

    println!("Load key successful");

    auth(&card, block);

    let read_apdu = [
        0xFF, // Class
        0xB0, // INS: Read Binary
        0x00, // P1: Offset (high order)
        block, // P2: Offset (low order, block 1)
        0x10  // Le: Number of bytes to read (16 bytes for a block)
    ];

    let mut rapdu = [0; MAX_BUFFER_SIZE];
    card.transmit(&read_apdu, &mut rapdu).expect("Failed to transmit read APDU");

    if rapdu[16] != 0x90 {
        Vec::from(&rapdu[..16])
    } else {
        Vec::from(&rapdu[..16])
    }
}


/// Lit les données d'un secteur spécifique de la carte.
///
/// # Arguments
///
/// * `card` - Référence à l'objet `Card` représentant la connexion à la carte.
/// * `index` - Numéro du secteur à lire.
///
/// # Retourne
///
/// Un vecteur de vecteurs d'octets contenant les données lues pour chaque bloc du secteur.
///
/// # Explication
///
/// Cette fonction lit les données de tous les blocs d'un secteur spécifique de la carte à puce. Un secteur est généralement composé de 4 blocs.
///
/// 1. **Initialisation du vecteur de résultats** :
///    Un vecteur vide (`result`) est initialisé pour stocker les données lues de chaque bloc.
///
/// 2. **Lecture de chaque bloc du secteur** :
///    Une boucle itère sur les numéros de bloc dans le secteur spécifié (du bloc `index` au bloc `index + 3` inclus).
///    Pour chaque bloc, la fonction `read` est appelée pour lire les données du bloc correspondant, et les données lues sont ajoutées au vecteur de résultats.
///
/// 3. **Retour du vecteur de résultats** :
///    Le vecteur contenant les données lues pour chaque bloc du secteur est retourné.
///
/// # Exemple
///
/// ```
/// let card: Card = ...; // Code pour initialiser et connecter la carte
/// let sector_data = read_sector(&card, 4);
/// for block_data in sector_data {
///     println!("{:?}", block_data);
/// }
/// ```
///
/// # Notes
///
/// - Cette fonction suppose que chaque secteur contient exactement 4 blocs.
/// - La fonction `read` utilisée doit être définie pour lire les données d'un bloc spécifique et retourner un vecteur d'octets.
///
fn read_sector(card: &Card, index: u8) -> Vec<Vec<u8>> {
    let mut result = Vec::new();
    for index_block in index..index + 4 {
        result.push(read(&card, index_block));
    }

    result
}


/// Convertit une chaîne hexadécimale en un tableau d'octets.
///
/// # Arguments
///
/// * `hexa` - Chaîne hexadécimale à convertir. La chaîne doit contenir exactement 32 caractères hexadécimaux (16 octets).
///
/// # Retourne
///
/// Un tableau de 16 octets représentant la conversion de la chaîne hexadécimale.
///
/// # Explication
///
/// Cette fonction prend une chaîne hexadécimale et la convertit en un tableau de 16 octets. Le processus est le suivant :
///
/// 1. **Initialisation du tableau** :
///    Un tableau de 16 octets (`tableau`) est initialisé avec des zéros. Un index est également initialisé à 0.
///
/// 2. **Conversion par morceaux** :
///    La chaîne hexadécimale est convertie en bytes (`as_bytes`) et traitée par morceaux de 2 caractères (`chunks_exact(2)`)
///    car chaque paire de caractères hexadécimaux représente un seul octet.
///
/// 3. **Conversion de chaque morceau** :
///    Pour chaque morceau de 2 caractères :
///
///    - La paire de caractères est convertie en une chaîne UTF-8 (`std::str::from_utf8(chunk).unwrap()`).
///    - La chaîne hexadécimale est ensuite convertie en un octet (`u8::from_str_radix(hex_str, 16).unwrap()`).
///    - L'octet résultant est stocké dans le tableau à l'index correspondant.
///    - L'index est incrémenté pour le prochain octet.
///
/// 4. **Retour du tableau** :
///    Le tableau rempli de 16 octets est retourné.
///
/// # Exemple
///
/// ```
/// let hexa_string = String::from("0123456789ABCDEF0123456789ABCDEF");
/// let result = hexa_to_tableau(hexa_string);
/// assert_eq!(result, [0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF]);
/// ```
///
fn hexa_to_tableau(hexa: String) -> [u8; 16] {
    let mut tableau = [0u8; 16];
    let mut index = 0;

    for chunk in hexa.as_bytes().chunks_exact(2) {
        let hex_str = std::str::from_utf8(chunk).unwrap();
        let byte = u8::from_str_radix(hex_str, 16).unwrap();
        tableau[index] = byte;
        index += 1;
    }

    tableau
}


/// Écrit des données dans un bloc spécifique de la carte.
///
/// # Arguments
///
/// * `card` - Référence à un objet `Card` représentant la connexion avec la carte à puce.
/// * `block` - Numéro du bloc de la carte où les données doivent être écrites. Notez que le bloc 4 et les blocs suivants peuvent être écrits.
/// * `data` - Tableau de 16 octets contenant les données à écrire dans le bloc spécifié.
///
/// # Explication
///
/// Cette fonction utilise un APDU (Application Protocol Data Unit) de commande pour écrire 16 octets de données
/// dans le bloc spécifié de la carte. Le processus est le suivant :
///
/// 1. **Vérification du bloc** :
///    Si le numéro du bloc est inférieur à 4, la fonction imprime un message indiquant que le secteur 0 (les blocs 0 à 3)
///    ne peut pas être modifié et retourne immédiatement, empêchant ainsi toute modification.
///
/// 2. **Préparation de l'APDU de commande d'écriture** :
///    L'APDU de commande est préparé avec les champs suivants :
///    - `0xFF` : Classe de commande.
///    - `0xD6` : Instruction pour écrire des données binaires.
///    - `0x00` : P1 (offset supérieur).
///    - `block` : P2 (offset inférieur, le numéro du bloc).
///    - `0x10` : Nombre d'octets à écrire (16 octets pour un bloc).
///    - `data[0]` à `data[15]` : Les 16 octets de données à écrire.
///
/// 3. **Transmission de l'APDU** :
///    La commande APDU préparée est transmise à la carte à puce en utilisant la méthode `transmit`.
///    La réponse de la carte est stockée dans le tableau `rapdu`.
///
/// 4. **Vérification du statut de l'opération** :
///    La fonction vérifie le code de statut de la réponse APDU.
///    - Si `rapdu[16]` (le 17ème octet) n'est pas égal à `0x90` (indiquant une opération réussie), elle imprime "Success".
///    - Sinon, elle imprime "Failed".
///
/// # Remarques
///
/// - `MAX_BUFFER_SIZE` doit être défini quelque part dans votre code pour représenter la taille maximale du tampon pour la réponse APDU.
/// - Le code vérifie l'octet 16 de la réponse APDU, mais souvent le code de statut est dans les deux derniers octets de la réponse.
///   Cela pourrait être une erreur potentielle dans la vérification du statut, à moins que cela ne soit conforme aux spécifications de votre carte particulière.
///
fn write(card: &Card, block: u8, data: [u8; 16]) {
    if block < 4 {
        println!("Le secteur 0 ne peut pas être modifié");
        return;
    }

    let write_apdu = [
        0xFF, // Class
        0xD6, // INS: Write Binary
        0x00, // P1: Offset (high order)
        block, // P2: Offset (low order, block 1)
        0x10,  // Le: Number of bytes write (16 bytes for a block)
        data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
        data[8], data[9], data[10], data[11], data[12], data[13], data[14], data[15]
    ];

    let mut rapdu = [0; MAX_BUFFER_SIZE];
    card.transmit(&write_apdu, &mut rapdu).expect("Failed to transmit write APDU");

    if rapdu[16] != 0x90 {
        println!("Success")
    } else {
        println!("Failed")
    }
}


/// Convertit un vecteur d'octets hexadécimaux en une valeur décimale.
///
/// # Arguments
///
/// * `tableau` - Vecteur d'octets hexadécimaux à convertir.
///
/// # Retourne
///
/// Une valeur entière de type `i128`.
///
/// # Explication
///
/// Cette fonction prend un vecteur d'octets représentant une valeur hexadécimale et la convertit en une valeur décimale de type `i128`. Le processus de conversion est le suivant :
///
/// 1. **Initialisation du résultat** :
///    Une variable `resultat` de type `i128` est initialisée à zéro pour stocker la valeur décimale résultante.
///
/// 2. **Conversion de chaque octet hexadécimal** :
///    Une boucle itère sur chaque élément (octet) du vecteur `tableau`.
///    - Pour chaque octet, il est décalé vers la gauche de 8 bits (multiplié par 256) et le résultat est mis à jour en ajoutant l'octet converti en décimal.
///
/// 3. **Retour de la valeur décimale** :
///    Une fois tous les octets traités, la fonction retourne la valeur décimale résultante.
///
/// # Exemple
///
/// ```
/// let hex_tableau = vec![0xFF, 0x01, 0x23];
/// let decimal_value = hexa_to_decimal(hex_tableau);
/// println!("{}", decimal_value); // Affiche "16776995"
/// ```
///
fn hexa_to_decimal(tableau: Vec<u8>) -> i128 {
    let mut resultat: i128 = 0;

    for &hex in tableau.iter() {
        resultat = (resultat << 8) | hex as i128;
    }

    resultat
}


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

