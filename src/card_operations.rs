/// Module `card_operations` fournit des fonctions pour gérer les opérations sur une carte à puce
/// à l'aide de la bibliothèque PCSC.
pub mod card_operations {
    use pcsc::*;

    /// Structure `CardManager` gère les opérations sur une carte.
    pub struct CardManager {
        pub card: Card,
    }

    impl CardManager {
        /// Charge le lecteur de carte et se connecte à la carte.
        ///
        /// # Retourne
        ///
        /// * `Ok(Card)` - Si la connexion est réussie.
        /// * `Err(Error)` - Si une erreur se produit lors de la connexion.
        ///
        /// # Exemples
        ///
        /// ```
        /// let card = CardManager::loadreader()?;
        /// ```

        pub fn loadreader() -> Result<Card, Error> {
            let ctx = Context::establish(Scope::User).expect("Etablissement du context échoué");

            let mut readers_buf = [0; 2048];
            let mut readers = match ctx.list_readers(&mut readers_buf) {
                Ok(readers) => readers,
                Err(err) => {
                eprintln!("Aucun lecteur trouvé: {}", err);
                std::process::exit(1);
                }
            };

            let reader = match readers.next() {
                Some(reader) => reader,
                None => { panic!("Pas de lecteur connecté"); }
                };

            match ctx.connect(reader, ShareMode::Shared, Protocols::ANY){
                Ok(a) => Ok(a),
                Err(E) => Err(E)
            }
        }

        /// Charge une clé dans la carte.
        ///
        /// # Arguments
        ///
        /// * `key` - Un tableau de 6 octets représentant la clé à charger.
        ///
        /// # Retourne
        ///
        /// * `u8` - 1 si le chargement de la clé réussit, 0 sinon.
        ///
        /// # Exemples
        ///
        /// ```
        /// card_manager.keyload([0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
        /// ```
        pub fn keyload(&self, key: [u8; 6]) -> u8 {
            let load_key_apdu = [
                0xFF, // Class
                0x82, // INS: Load Authentication Key
                0x00, // P1: Key Structure
                0x00, // P2: Key Slot (0)
                0x06, // Lc: Length of Key
                key[0], key[1], key[2], key[3], key[4], key[5] // Key A
            ];

            let mut rapdu = [0; 256];
             match self.card.transmit(&load_key_apdu, &mut rapdu) {
                Ok(v) => {
                    return 1;
                }
                Err(e) => {
                    println!("Erreur lors de la transmission: {:?}", e);
                }
             }
            let status_word = &rapdu[..2];
            if status_word != [0x90, 0x00] {
                eprintln!("Chargement des clés échouées, code: {:02X?}", status_word);
                return 0;
            }

            return 1;
        }

        /// Authentifie la carte pour un bloc spécifique.
        ///
        /// # Arguments
        ///
        /// * `block` - Le numéro du bloc à authentifier.
        ///
        /// # Retourne
        ///
        /// * `u8` - 1 si l'authentification réussit, 0 sinon.
        ///
        /// # Exemples
        ///
        /// ```
        /// card_manager.auth(4);
        /// ```
        pub fn auth(&self, block: u8) -> u8 {
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
            let mut rapdu = [0; 256];
            self.card.transmit(&auth_apdu, &mut rapdu).expect("Authentification de la carte échouée");

            let status_word = rapdu[0];
            if status_word != 0x90 {
                eprintln!("Authentification échouée, code: {:02X?}", status_word);
                return 0;
            }

            return 1;

        }

        /// Lit les données d'un bloc spécifique.
        ///
        /// # Arguments
        ///
        /// * `block` - Le numéro du bloc à lire.
        ///
        /// # Retourne
        ///
        /// * `Vec<u8>` - Les données lues du bloc.
        ///
        /// # Exemples
        ///
        /// ```
        /// let data = card_manager.read(4);
        /// ```
        pub fn read(&self, block: u8) -> Vec<u8> {
            self.keyload([0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);

            self.auth(block);

            let read_apdu = [
                0xFF, // Class
                0xB0, // INS: Read Binary
                0x00, // P1: Offset (high order)
                block, // P2: Offset (low order, block 1)
                0x10  // Le: Number of bytes to read (16 bytes for a block)
            ];

            let mut rapdu = [0; 256];
            self.card.transmit(&read_apdu, &mut rapdu).expect("Failed to transmit read APDU");

            if rapdu[16] != 0x90 {
                Vec::from(&rapdu[..16])
            } else {
                Vec::from(&rapdu[..16])
            }
        }

        /// Écrit des données dans un bloc spécifique.
        ///
        /// # Arguments
        ///
        /// * `block` - Le numéro du bloc à écrire.
        /// * `data` - Un tableau de 16 octets représentant les données à écrire.
        ///
        /// # Exemples
        ///
        /// ```
        /// card_manager.write(4, [0x00; 16]);
        /// ```
        pub fn write(&self, block: u8, data: [u8; 16]) {
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

            let mut rapdu = [0; 256];
            self.card.transmit(&write_apdu, &mut rapdu).expect("Failed to transmit write APDU");

            if rapdu[16] != 0x90 {
                println!("Success")
            } else {
                println!("Failed")
            }
        }


        /// Lit les données d'un secteur spécifique.
        ///
        /// # Arguments
        ///
        /// * `sector` - Le numéro du secteur à lire.
        ///
        /// # Retourne
        ///
        /// * `Vec<Vec<u8>>` - Les données lues du secteur.
        ///
        /// # Exemples
        ///
        /// ```
        /// let data = card_manager.read_sector(1);
        /// ```
        pub fn read_sector(&self, sector: u8) -> Vec<Vec<u8>> {
            let mut blocks = Vec::new();
            let start_block = sector * 4;

            for block_offset in 0..3 {
                let block = start_block + block_offset;
                blocks.push(self.read(block));
            }

            blocks
        }

        /// Écrit des données dans un secteur spécifique.
        ///
        /// # Arguments
        ///
        /// * `sector` - Le numéro du secteur à écrire.
        /// * `data` - Un vecteur de tableaux de 16 octets représentant les données à écrire.
        ///
        /// # Exemples
        ///
        /// ```
        /// let data = vec![[0x00; 16]; 3];
        /// card_manager.write_sector(1, data);
        /// ```
        pub fn write_sector(&self, sector: u8, data: Vec<[u8; 16]>) {
            let start_block = sector * 4;

            for (block_offset, block_data) in data.iter().enumerate() {
                let block = start_block + block_offset as u8;
                self.write(block, *block_data);
            }
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn test_loadreader_with_card() {
            let card = CardManager::loadreader().unwrap();
            assert!(card.is_connected());
        }

        #[test]
        fn test_loadreader_without_card() {
            let card = CardManager::loadreader().unwrap();
            assert!(!card.is_connected());
        }

        #[test]
        fn test_keyload_valid() {
            let card = CardManager::loadreader().unwrap();
            let res = card.keyload([0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
            assert_eq!(res, 1);
        }

        #[test]
        fn test_keyload_invalid() {
            let card = CardManager::loadreader().unwrap();
            let res = card.keyload([0x0f, 0xff, 0xef, 0xef, 0xaf, 0xff]);
            assert_eq!(res, 0);
        }

        fn test_read() -> Vec<Vec<u8>>{
            let card = CardManager::loadreader().unwrap();
            let data = card.read_sector(3);
            data
        }

        fn test_write(){
            let data = [0x00; 16];
            let card = CardManager::loadreader().unwrap();
            card.write_sector(3, data);

        }

        #[test]
        fn test_write_and_read() {
            test_write();
            let data = test_read();
            assert_eq!(data, vec![[0x00; 16]; 3]);
        }


    }


}
