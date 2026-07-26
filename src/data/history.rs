// Gestion de l'historique des undo/redo.
// Ce module est nécessaire car le mode contrôlé gérant les inputs dans la forme
// casse les undo/redo natifs du browser. Ce module agit sur les signaux ce qui met
// automatiquement à jour les champs affichés ainsi que les dépendances inter-champs
// (par la magie d'un use_effect)

use dioxus::prelude::*;

// Taille maximum de l'historique.
// Même si chaque entrée est petite, un utilisateur qui joue longtemps peut accumuler des milliers d'entrées.
// Il convient donc de limiter sa taille.
const MAX_HISTORY: usize = 500;

// Entrée de l'historique avec l'id, le signal, l'ancienne valeur et la nouvelle valeur.
#[derive(Clone, Copy)]
enum HistoryEntry {
    Int {
        id: &'static str,
        signal: WriteSignal<i32>,
        old_value: i32,
        new_value: i32,
    },
    Bool {
        id: &'static str,
        signal: WriteSignal<bool>,
        old_value: bool,
        new_value: bool,
    },
}

impl HistoryEntry {
    // Postionne le focus sur l'élément annulé ou rétabli
    fn set_focus(id: &'static str) {
        // Commande javascript positionnant le focus sur cet élément
        let js = format!(r#"document.getElementById("{id}").focus();"#);
        // Execution de cette commande javascript
        spawn(async move {
            let eval = document::eval(&js);
            let _ = eval.await;
        });
    }

    // L'undo réecrit l'ancienne valeur dans le signal
    fn undo(&self) {
        match self {
            Self::Int {
                id,
                mut signal,
                old_value,
                ..
            } => {
                signal.set(*old_value);
                Self::set_focus(id);
            }
            Self::Bool {
                id,
                mut signal,
                old_value,
                ..
            } => {
                signal.set(*old_value);
                Self::set_focus(id);
            }
        }
    }

    // Le redo réecrit la nouvelle valeur dans le signal
    fn redo(&self) {
        match self {
            Self::Int {
                id,
                mut signal,
                new_value,
                ..
            } => {
                signal.set(*new_value);
                Self::set_focus(id);
            }
            Self::Bool {
                id,
                mut signal,
                new_value,
                ..
            } => {
                signal.set(*new_value);
                // Même s'il n'y a pas d'indication apparente du focus sur un checkbox,
                // cela permet de faire scroller la fenêtre pour le rendre visible.
                Self::set_focus(id);
            }
        }
    }
}

// Historique avec une pile pour les undo et une autre pour les redo
#[derive(Default, Clone, Store)]
pub struct History {
    undo_stack: Vec<HistoryEntry>,
    redo_stack: Vec<HistoryEntry>,
}

impl History {
    pub fn new() -> Self {
        Self::default()
    }

    // Pour les nombres, l'ancienne valeur est explicitement passée en paramètre
    // et n'est pas récupérée depuis le signal car celui-ci a subit des écritures
    // intermédiaires dans le oninput.
    pub fn add_i32(
        &mut self,
        id: &'static str,
        mut signal: WriteSignal<i32>,
        old_value: i32,
        new_value: i32,
    ) {
        // La nouvelle valeur est écrite systématiquement même si old_val == new_val
        // car il a pu y avoir des écritures intermédiaire ayant modifié le signal
        // depuis l'entrée dans le champ.
        signal.set(new_value);

        // Par contre une nouvelle entrée n'est ajoutée dans la pile des undo que si la valeur a changé
        if old_value == new_value {
            return;
        }

        self.undo_stack.push(HistoryEntry::Int {
            id,
            signal,
            old_value,
            new_value,
        });

        // Limite la taille de la pile au 500 derniers éléments.
        if self.undo_stack.len() > MAX_HISTORY {
            self.undo_stack.remove(0);
        }

        // Une nouvelle entrée vide la pile des redo
        self.redo_stack.clear();
    }

    // Pour les booléens, l'ancienne valeur est récupérée depuis le signal
    // car celui-ci n'a pas subit d'écritures intermédiaires.
    pub fn add_bool(&mut self, id: &'static str, mut signal: WriteSignal<bool>, new_value: bool) {
        let old_value = signal();

        if old_value == new_value {
            return;
        }

        signal.set(new_value);

        self.undo_stack.push(HistoryEntry::Bool {
            id,
            signal,
            old_value,
            new_value,
        });

        if self.undo_stack.len() > MAX_HISTORY {
            self.undo_stack.remove(0);
        }

        self.redo_stack.clear();
    }

    // L'undo est relayé à l'entrée puis l'entrée est déplacée vers la pile des redo
    pub fn undo(&mut self) {
        if let Some(entry) = self.undo_stack.pop() {
            entry.undo();
            self.redo_stack.push(entry);
        }
    }

    // Le redo est relayé à l'entrée puis l'entrée est déplacée vers la pile des undo
    pub fn redo(&mut self) {
        if let Some(entry) = self.redo_stack.pop() {
            entry.redo();
            self.undo_stack.push(entry);
        }
    }
}
