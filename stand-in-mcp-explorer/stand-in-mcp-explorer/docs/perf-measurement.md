# Medição de performance das listas (037 / O-024)

Harness gated para diagnosticar a "rolagem dura" com listas grandes. O agente
instrumenta; **você** roda e rola; o agente analisa o JSONL e gera o backlog.

## Como rodar

A instrumentação fica atrás da feature **dev-only `perf`** (off por default — um
build normal não tem o código de medição nem a dep `sysinfo`). Rode com
`--features perf`, pela raiz do repo (workspace excluído → `--manifest-path`):

```sh
# Tools (uniform_list / altura fixa) — N tools sintéticos
cargo run --manifest-path stand-in-mcp-explorer/Cargo.toml --features perf \
  -p stand-in-mcp-explorer -- --capture tools perf

# History (list/ListState / altura variável) — N entradas sintéticas
cargo run --manifest-path stand-in-mcp-explorer/Cargo.toml --features perf \
  -p stand-in-mcp-explorer -- --capture history perf
```

Configuração por variável de ambiente:

- `MCPX_PERF_N` — tamanho da lista sintética (default 5000). Ex.: `MCPX_PERF_N=10000`.
- `MCPX_PERF_LOG` — caminho do JSONL (default `./mcpx-perf-<unixmillis>.jsonl`,
  impresso no startup como `PERF LOG: <caminho-absoluto>`).
- `MCPX_PERF=1` — liga a instrumentação **sem** o fixture (mede a lista que
  estiver em tela numa conexão real). O caminho recomendado é o fixture.

## Protocolo de medição

1. Abra com o comando acima. Anote o `PERF LOG:` impresso no terminal.
2. **Aguarde ~2 s** parado (captura a fase `init` — primeiro paint + amostras).
3. **Role continuamente** a lista por ~10 s (para baixo e para cima).
4. **Feche a janela.** O JSONL está completo (cada linha é flushed na escrita).

Rode N diferentes (1k / 5k / 10k / 50k) em execuções separadas para ver a escala.

## O que é logado (JSONL, um evento por linha)

`frame` — um por frame renderizado da lista ativa:

- `t_ms`, `frame` (índice monotônico), `tab`, `list`, `phase` (`init` | `scroll`)
- `scroll_ix` — índice do item no topo visível (posição da rolagem)
- `dt_ms` — intervalo desde o frame anterior (fluidez; **jank** = `dt_ms` > 16.7)
- `visible`, `not_rendered`, `total` — contagens de itens (renderizados ×
  janela × total)
- `build_us`, `build_per_item_us` — custo de construir os elementos **visíveis**
  (no closure de windowing)
- **`clone_us`** — custo do clone O(N) do snapshot por frame (O-013)
- **`render_us`** — custo do corpo de `render()` (clones + build eager da árvore;
  o build dos itens visíveis é deferido para o layout, então NÃO entra aqui)

**Atribuição (O-027):** `dt_ms` é o frame inteiro (render + layout + paint + idle);
`render_us` é só o corpo do `render`; `clone_us` é só o clone. Se `render_us` ≈
`dt_ms`, o gargalo é o corpo do render (e `clone_us` diz quanto é clone); se
`render_us` ≪ `dt_ms`, é o layout/paint do gpui.

`sample` — periódico (~200 ms, thread `sysinfo`):

- `t_ms`, `phase`, `cpu_pct` (% — pode passar de 100 em multi-core), `rss_bytes`

## Notas / caveats

- **Fase:** o `phase` in-app deriva de `scroll_ix` (índice real do topo, lido do
  scroll-handle): `scroll` quando `scroll_ix > 0`, senão `init`. Imune à passada
  de medição do `uniform_list` (que chamava o closure com range `0..1`). A
  segmentação fina pode ser refeita na análise por `scroll_ix` / `dt_ms` / `t_ms`.
- **Build ≠ paint:** o GPUI não expõe paint por item; `build_*` mede o tempo de
  construir os elementos do frame (a parte sob nosso controle). A fluidez
  (`dt_ms`, intervalo render→render) cobre o ciclo observável.
- **Overhead:** desligado, os hooks custam um load atômico + branch. Ligado,
  cada frame paga um lock + uma linha formatada à mão + write bufferizado e
  flushed — registrado aqui como caveat da medição.
- **CPU/mem:** amostrados do próprio processo via `sysinfo`; a primeira amostra
  de CPU pode vir 0 (sem baseline) e a do init costuma saturar (build da lista +
  primeiro paint).
