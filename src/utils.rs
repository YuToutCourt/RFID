pub mod utils {

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
    pub fn hexa_to_tableau(hexa: String) -> [u8; 16] {
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
    pub fn hexa_to_decimal(tableau: Vec<u8>) -> i128 {
        let mut resultat: i128 = 0;

        for &hex in tableau.iter() {
            resultat = (resultat << 8) | hex as i128;
        }

        resultat
    }
}
