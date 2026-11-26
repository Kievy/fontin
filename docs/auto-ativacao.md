# Auto-ativação de fontes — especificação

Status: **proposta** · Escopo fechado: Figma, Adobe (Photoshop, Premiere Pro, After Effects, Illustrator) e ArtCraft (PhotoCraft, EffectCraft, FilmCraft, VectorCraft).

## Objetivo

Quando um documento aberto em um desses apps usa uma fonte que não está ativa, o Fontin:

1. procura a fonte nas **pastas da biblioteca** → ativa;
2. senão, procura no **Google Fonts** → baixa para o cache e ativa;
3. senão, mostra uma **notificação do sistema** listando as fontes não encontradas (e registra no histórico do app).

Não existe um evento do sistema operacional para "fonte pedida e ausente" — cada app só cai numa fonte substituta em silêncio. Por isso cada pacote tem um **sensor** próprio que descobre as fontes do documento e as envia ao Fontin.

## Arquitetura

```
 Figma plugin ─┐
 Adobe (UXP/CEP)├──► Fontin Link (HTTP 127.0.0.1) ──► Resolver ──► Activator
 ArtCraft ─────┘         token + origem                 │
                                                       └──► Notificação (faltando)
```

### Fontin Link (núcleo, comum aos três)

- Servidor HTTP só em `127.0.0.1`, porta fixa preferencial (ex.: `43170`) com fallback; porta e token gravados em `~/.local/share/app.fontin.desktop/link.json` (equivalente no Windows/macOS).
- Autenticação: header `Authorization: Bearer <token>`. O token é gerado na primeira execução; o plugin do Figma recebe um código de pareamento exibido nas Configurações (o Figma não lê arquivos locais).
- Ligável/desligável em **Configurações → Auto-ativação**, com uma chave por pacote.

Endpoints:

| Método | Rota | Corpo | Resposta |
| --- | --- | --- | --- |
| `GET` | `/v1/ping` | — | versão do Fontin |
| `POST` | `/v1/resolve` | `{ app, document, fonts: [{ family, style?, postscript? }] }` | `{ activated: [...], alreadyActive: [...], downloaded: [...], missing: [...] }` |

### Resolver

Ordem de correspondência para cada fonte pedida:

1. **PostScript name** exato (Adobe usa isso: `Roboto-BoldItalic`).
2. **Família + estilo** (normalizados: minúsculas, sem espaços/hífens; `Bold Italic` ≈ `BoldItalic`).
3. **Só família** → ativa a família inteira (cobre apps que pedem só o nome).
4. Catálogo do **Google Fonts** pela família → download + ativação.
5. Não achou → `missing`.

Regras:
- Respeita fontes já ativas e instaladas no sistema (não duplica).
- Dedup de pedidos: o mesmo documento não dispara notificações repetidas por 10 min.
- Notificação agrupada: "3 fontes não encontradas em *cartaz.psd*: Helvetica Now, …" com ação "Abrir Fontin".
- Histórico em **Biblioteca → Auto-ativação** (o que foi ativado, por qual app, quando).

## Sensores por pacote

### Figma

| Item | Detalhe |
| --- | --- |
| Tecnologia | Plugin do Figma (TypeScript), `networkAccess` liberado para `http://localhost` |
| Detecção | Percorre os nós de texto (`figma.loadAllPagesAsync` + `node.hasMissingFont`, `getStyledTextSegments(['fontName'])`) |
| Gatilho | **Limitação do Figma:** plugins não rodam sozinhos ao abrir um arquivo. O usuário executa "Fontin → Ativar fontes do arquivo" (1 clique; pode ficar aberto e reagir a `documentchange`). |
| Depois de ativar | O agente de fontes do Figma precisa recarregar a lista; o plugin avisa "reabra o arquivo se a fonte ainda aparecer como ausente". |
| Plataformas | Windows, macOS; Linux só via clientes não oficiais (figma-linux), a validar. |

### Adobe

Uma extensão por app (UXP onde já é suportado, CEP/ExtendScript onde não é). Fontes Adobe são referenciadas por **PostScript name**, o que dá correspondência exata.

| App | API | Como lê as fontes | Gatilho | Confiança |
| --- | --- | --- | --- | --- |
| Illustrator | CEP + ExtendScript | `textFrames[].textRange.characterAttributes.textFont` e metadados XMP `xmpTPg:Fonts` do documento | evento `documentAfterActivate` | Alta |
| Photoshop | UXP | `textItem.characterStyle.font` das camadas de texto | evento `open` / `select` de documento | Alta |
| After Effects | CEP + ExtendScript | `TextDocument.font` / `fontFamily` / `fontStyle`; AE 24+: `app.fonts.missingOrSubstitutedFonts` | painel observa troca de projeto (`app.project.file`) | Média–alta |
| Premiere Pro | UXP (25+) | Textos de gráficos/legendas — a API de leitura de fontes é limitada | abertura de projeto | **Baixa — investigar** |

Observações:
- Adobe só roda em **Windows e macOS** → desenvolvimento e teste nessas plataformas.
- Fontes ativadas via GDI/CoreText são vistas pelos apps Adobe sem reiniciar, na maioria dos casos (Photoshop pode exigir reabrir o documento).
- Fontes da Adobe Fonts (Creative Cloud) ficam de fora: já são gerenciadas pela Adobe.
- Distribuição: pacote `.ccx` (UXP) / `.zxp` (CEP), instalado pelo próprio Fontin em **Configurações → Integrações**.

### ArtCraft (storytold)

Apps open source em Rust/egui, multiplataforma (Windows, macOS, Linux, FreeBSD). Sem sistema de plugins; usam fontes embutidas (`craft-fonts`) e caem nas **fontes do sistema**.

| App | Arquivo | Formato |
| --- | --- | --- |
| VectorCraft | `.vectorcraft` | JSON |
| EffectCraft | `.ecproj` | JSON versionado |
| PhotoCraft | `.pcraft` (+ PSD/PSB) | a confirmar (`book/`) |
| FilmCraft | `.fcproj` | a confirmar (`docs/project-files.md`) |

Problema: apps Rust costumam carregar a lista de fontes do sistema **uma vez, ao iniciar**. Ativar depois que o app abriu pode não ter efeito até reiniciar. Por isso a estratégia é **ativar antes de abrir**:

1. **"Abrir com Fontin" (principal):** o Fontin se registra como abridor dos formatos acima. Ao abrir um arquivo ele lê as fontes do projeto, resolve/ativa e então lança o app correspondente com o arquivo. Transparente para o usuário depois de configurado.
2. **Canal de controle (complementar):** os apps têm um canal opcional JSON em loopback (`--control <porta>`, ex.: EffectCraft `9877`, FilmCraft `9886`). Se o protocolo expuser o documento aberto, o Fontin pode consultar e ativar com o app já aberto.
3. **Integração nativa (longo prazo):** propor ao projeto (crate `craft-libs`) um gancho que chame o `/v1/resolve` do Fontin quando uma fonte não for encontrada, e que recarregue as fontes do sistema em tempo de execução.

Pendências de investigação: estrutura dos campos de fonte em cada formato; `docs/control-protocol.md` (VectorCraft) e equivalentes; backend de fontes no `Cargo.toml` de cada app (fontdb/cosmic-text?) e se recarrega fontes em execução.

## Fases

| Fase | Entrega | Testável em |
| --- | --- | --- |
| 1 | Fontin Link + Resolver + notificações + histórico + configurações | Linux |
| 2 | Plugin do Figma | Windows/macOS (Linux via cliente não oficial) |
| 3 | ArtCraft: leitura dos formatos + "Abrir com Fontin" | Linux |
| 4 | Adobe: Illustrator e Photoshop | Windows/macOS |
| 5 | Adobe: After Effects; investigação do Premiere | Windows/macOS |
