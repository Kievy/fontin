<img src="assets/icon.svg" width="64" height="64" alt="">

# Fontin

Gerenciador de fontes open source. A fonte entra no sistema **sem ser instalada**: liga e desliga.

- **Ativação sem instalar** — nada é copiado para pastas do sistema; o Fontin só registra uma referência ao arquivo original.
- **Google Fonts integrado** — navegue pelo catálogo completo com prévia ao vivo (streaming, sem download); ao ativar, os arquivos vão para o cache do app e ficam disponíveis em qualquer programa.
- **Rápido** — núcleo em Rust: varredura paralela com `rayon` + `memmap2` + `ttf-parser`, índice em memória e rescans incrementais (só arquivos alterados são relidos).

## Stack

| Camada | Tecnologia |
| --- | --- |
| Núcleo | Rust (Tauri v2), SQLite (`rusqlite`, bundled) |
| UI | Svelte 5 + SvelteKit (SPA estática), TypeScript |
| Ícones / tipografia da UI | Lucide, Inter Variable, Urbanist Variable |

## Como a ativação funciona

| SO | Mecanismo | Escopo |
| --- | --- | --- |
| Linux | Symlinks em `~/.local/share/app.fontin.desktop/active`, registrado no fontconfig por `~/.config/fontconfig/conf.d/90-fontin.conf` + `fc-cache -f` (com debounce) | Usuário |
| Windows | `AddFontResourceW` + `WM_FONTCHANGE` | Sessão (reaplicado ao abrir o app) |
| macOS | `CTFontManagerRegisterFontsForURL` (`kCTFontManagerScopeSession`) | Sessão (reaplicado ao abrir o app) |

O estado ativo fica salvo no SQLite; ao iniciar, o Fontin reconcilia o sistema com o banco (`Activator::sync`).

## Estrutura

```
src-tauri/src/
  lib.rs            setup do Tauri, tarefas em segundo plano no start
  state.rs          AppState: biblioteca, banco, catálogo, ativador
  commands.rs       comandos IPC + eventos (library-changed, download-progress…)
  model.rs          tipos do domínio e tipos enviados à UI
  scanner.rs        varredura paralela e incremental de pastas
  library.rs        índice em memória: famílias, filtros, árvore de pastas
  db.rs             persistência SQLite
  google.rs         catálogo e download do Google Fonts
  activation/       Activator por plataforma (linux, windows, macos)

src/lib/
  api.ts            wrappers tipados do IPC
  store.svelte.ts   estado global (runes), eventos do backend
  fonts.ts          carregamento sob demanda das prévias (LRU)
  styles/app.css    tokens de design (grid de 8 px)
  components/       Rail, Sidebar, Header, FontList (virtualizada), FontRow,
                    DetailPanel, Toggle (o interruptor da marca)…
```

## Desenvolvimento

Pré-requisitos: Rust estável, Node 20+, e as [dependências do Tauri](https://tauri.app/start/prerequisites/) para o seu SO.

```sh
npm install
npm run tauri dev       # app em modo desenvolvimento
npm run tauri build     # pacote de release
npm run check           # checagem de tipos do frontend

cd src-tauri
cargo test              # testes (inclui varredura de /usr/share/fonts)
cargo test -- --ignored # testes com rede (catálogo e download do Google)
```

## Licença

MIT
