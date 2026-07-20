# CRM Builder — Όραμα & Τρέχουσα Κατάσταση

## 1. Η Μεγάλη Εικόνα

Μια **CRM πλατφόρμα** φτιαγμένη σε **Rust + egui** (native desktop, cross-platform).
Στόχος: να επιτρέπει σε non-technical χρήστες να σχεδιάζουν custom CRM διεπαφές
(φόρμες, πίνακες, ροές) μέσα από ένα visual editor — σαν να είναι Figma / Penpot
για CRM.

### Πυρήνας (crm-core)
- Database SQLite με `Database`, `Project`, `Record`, `FlowGraph`, `FlowNode`, `FlowEdge`
- Αποθήκευση/φόρτωση project, records, flows, επαφών, deals, tasks

### Desktop App (crm-ui)
- Single-window egui app με mode switching: **Designer, Contacts, Pipeline, Studio, Tasks**
- Dark theme Windows 11 / Linear.app hybrid
- Backend-agnostic: δουλεύει και offline με SQLite

---

## 2. Τι Έχουμε Χτίσει Μέχρι Τώρα

### 2.1 Designer Mode (Visual Editor)
| Χαρακτηριστικό | Κατάσταση |
|---|---|
| **Infinite canvas** με pan/zoom | ✅ |
| Scroll wheel → zoom (0.1x – 5x) | ✅ |
| Middle-mouse / Space+drag → pan | ✅ |
| **Scene graph** (ιεραρχικά nodes με parent/child) | ✅ |
| Z-order rendering | ✅ |
| Smart snapping (align/distances) | ✅ |
| Grid mode | ✅ |
| Widget palette (drag & drop στο canvas) | ✅ |
| Element selection + drag reposition | ✅ |
| **8-point resize handles** (γωνίες + ακμές) | ✅ |
| Right-click context menu (go to flows) | ✅ |
| Left panel: widget palette ή layers tree | ✅ |
| Right panel: transform + constraints + properties | ✅ |
| Serialization/deserialization v2 JSON | ✅ |
| Backward compatibility με v1 format | ✅ |

### 2.2 CRM Widgets (αντί για generic)
| Widget | Properties |
|---|---|
| **InputField** | label, placeholder, field type (text/email/phone/number/date/password), data binding, required |
| **TextArea** | label, placeholder, data binding, required |
| **Dropdown** | label, options, multiple, data binding, required |
| **Checkbox** | label, data binding |
| **Toggle** | label, data binding |
| **SearchBox** | placeholder, data binding |
| **Button** | label, style (primary/secondary/danger/ghost), action (none/submit/go back/open flow/navigate) |
| **Label** | text, font size, align |
| **DataTable** | entity binding, inline add/delete records |
| **Image** | URL |
| **Divider** | — |

### 2.3 Flow Builder
| Χαρακτηριστικό | Κατάσταση |
|---|---|
| Flow nodes (trigger/action/condition/gate) | ✅ |
| Edges με βέλη | ✅ |
| Drag reposition nodes | ✅ |
| Connection drawing (από output σε input) | ✅ |
| Context menu (delete node) | ✅ |
| Flow execution από Designer buttons | 🟡 (hardcoded) |

### 2.4 Λοιπές Λειτουργίες
| Module | Κατάσταση |
|---|---|
| **Contacts** (list, search, edit, notes, inline delete) | ✅ |
| **Pipeline/Deals** (kanban, drag between stages, edit) | ✅ |
| **Tasks** (list, filter by status/priority, edit, delete) | ✅ |
| **Studio** (vector layers: shapes, brush, text, eraser) | 🟡 βασικό |
| Save/Load project | ✅ |
| Toast notifications | ✅ |

### 2.5 Αρχιτεκτονική
- `CrmApp` struct — όλη η κατάσταση σε ένα struct (no Redux, no complexity)
- `Viewport2D` — world-to-screen transforms
- `ElementNode` — scene graph node με `local_rect`, `constraints`, `content`, `parent_id`
- `ElementType` — enum με όλα τα widget types
- `Mode` enum — Designer | Contacts | Pipeline | Studio | Tasks
- Serialization: manual `serde_json::json!()` με v2 schema

---

## 3. Τι Θέλουμε να Χτίσουμε (Μελλοντικά)

### 3.1 Designer — Short-term
- [ ] **Duplicate, delete, copy/paste** στοιχεία (shortcuts: Ctrl+D, Delete, Ctrl+C/V)
- [ ] **Multi-select** (Shift+click ή marquee selection)
- [ ] **Group/Ungroup** στοιχεία (parent/child μέσω context menu)
- [ ] **Lock/unlock, visibility toggle** από layers panel
- [ ] **Undo/Redo** (command stack)
- [ ] **Keyboard shortcuts** (Delete, Escape, Ctrl+A, arrows for nudge)
- [ ] **Alignment tools** (align left/right/center/top/bottom από toolbar)
- [ ] **Distribute tools** (even horizontal/vertical spacing)

### 3.2 CRM Binding — Real Data
- [ ] **Data source picker**: στο right panel, bind widgets σε real CRM entities/fields
- [ ] **Live preview**: δες real data μέσα στον designer (όχι mock)
- [ ] **Form mode**: τα widgets λειτουργούν σαν φόρμα (input → save σε DB)
- [ ] **DataTable sorting/filtering** runtime
- [ ] **Dynamic dropdowns** από DB values

### 3.3 Flow Builder — Επόμενο Επίπεδο
- [ ] **Flow node types**: Send Email, Create Record, Update Record, Webhook, Delay, Condition (if/else)
- [ ] **Flow testing** (run flow with mock data, see trace)
- [ ] **Visual feedback** (success/error states σε nodes)
- [ ] **Sub-flows** (call one flow from another)
- [ ] **Flow templates**

### 3.4 Contacts
- [ ] **Import από CSV/Excel**
- [ ] **Bulk actions** (delete, tag, export)
- [ ] **Activity timeline** (notes, calls, emails)
- [ ] **Contact merging** (dedup)

### 3.5 Pipeline / Deals
- [ ] **Custom stages** (ο χρήστης ορίζει τα stages)
- [ ] **Drag deals between stages** (kanban drag)
- [ ] **Deal value forecasting** (simple chart)
- [ ] **Won/Lost reasons**

### 3.6 Studio Mode
- [ ] **Text layers** πλήρης επεξεργασία
- [ ] **Layer reorder** (drag up/down)
- [ ] **Export layers** ως PNG/SVG
- [ ] **Zoom independent από designer zoom**

### 3.7 Platform / Infrastructure
- [ ] **Authentication** (Google OAuth + demo mode)
- [ ] **Cloud sync** (REST API server — υπάρχει license-server)
- [ ] **Multi-project support** (project list / switch)
- [ ] **Auto-save**
- [ ] **Plugin system** (WASM plugins)

### 3.8 UI Polish
- [ ] **Context toolbars** (context-sensitive actions πάνω από canvas)
- [ ] **Mini-map** (birds-eye view του canvas)
- [ ] **Snap to grid toggle** (όχι μόνο grid mode)
- [ ] **Ruler guides** (σύρσιμο guides από τα περιθώρια)
- [ ] **Dark/light theme toggle**
- [ ] **Font picker** για labels

---

## 4. Τρέχοντα Active Items

- ✅ Designer — resize handles
- ✅ Designer — right-click context menu (go to modules)
- ⬜ Flow — flow execution με πραγματικά events
- ⬜ Designer — keyboard shortcuts + undo/redo
- ⬜ Data binding — live preview
- ⬜ Multi-project support

---

## 5. Σημειώσεις Αρχιτεκτονικής

- **No premature abstraction**: όλη η κατάσταση στο `CrmApp`, καμία επιπλέον indirection
- **Widget rendering**: manual `egui::Painter` calls, όχι widgets
- **Serialization**: manual `serde_json::json!()` (μέγιστος έλεγχος, μηδέν dependencies)
- **DB schema**: crm-core διαχειρίζεται SQLite, crm-ui μόνο data structures
- **Προσοχή σε over-engineering**: κάθε feature προστίθεται μόνο όταν χρειαστεί
