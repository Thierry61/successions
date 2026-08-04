// Séparation de l'application en 2 (bin + lib) pour pouvoir définir des tests d'intégration
// (ce qui n'est pas possible dans un crate purement bin)

fn main() {
    dioxus::launch(successions::App);
}
