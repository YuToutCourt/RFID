pub mod card_operations {
    use pcsc::*;

    pub struct CardManager {
        card: Card,
    }

    impl CardManager {
        pub fn new(card: Card) -> Self {
            Self { card }
        }

        
        pub fn keyload(&self, key: [u8; 6]) {
            let load_key_apdu = [
                0xFF, // Class
                0x82, // INS: Load Authentication Key
                0x00, // P1: Key Structure
                0x00, // P2: Key Slot (0)
                0x06, // Lc: Length of Key
                key[0], key[1], key[2], key[3], key[4], key[5] // Key A
            ];

            let mut rapdu = [0; 256];
            self.card.transmit(&load_key_apdu, &mut rapdu).expect("Impossible de charger les clés");

            let status_word = &rapdu[..2];
            if status_word != [0x90, 0x00] {
                eprintln!("Load key failed with status: {:02X?}", status_word);
                return;
            }
        }

        pub fn auth(&self, block: u8) {
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
            self.card.transmit(&auth_apdu, &mut rapdu).expect("Failed to authenticate");

            let status_word = rapdu[0];
            if status_word != 0x90 {
                eprintln!("Auth failed: {:02X?}", status_word);
                return;
            }
        }

        pub fn read(&self, block: u8) -> Vec<u8> {
            self.keyload([0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);

            println!("Load key successful");

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
                return Vec::from(&rapdu[..16]);
            } else {
                return Vec::from(&rapdu[..16]);
            }
        }

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

        pub fn read_sector(&self, sector: u8) -> Vec<Vec<u8>> {
            let mut blocks = Vec::new();
            let start_block = sector * 4;

            for block_offset in 0..3 {
                let block = start_block + block_offset;
                blocks.push(self.read(block));
            }

            blocks
        }

        pub fn write_sector(&self, sector: u8, data: Vec<[u8; 16]>) {
            let start_block = sector * 4;

            for (block_offset, block_data) in data.iter().enumerate() {
                let block = start_block + block_offset as u8;
                self.write(block, *block_data);
            }
        }
    }
}
