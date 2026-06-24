# Changelog — stand-in-mcp-explorer-ds

> Alterações que afetam o contrato do Design System. O crate é `publish = false`;
> este changelog registra evoluções de token/componente para o app consumidor.

## [Em desenvolvimento]

- `CountPill` (036 M1): pill numérico redondo para contagens inline (ex.: env
  vars). `RenderOnce`, bg `palette::OBY` opaco, fg branco, `FS_2XS` / `W_SEMIBOLD`
  / mono, `RADIUS_PILL` (99), min_w(18), h(18). API: `CountPill::new(n)` +
  `.id(…)`. Não interativo. Retrocompatível (componente novo).

- `Field::secret()` (035 M3): input mascarado com toggle olho via
  `InputState::masked` + ícone `Eye` no `.suffix`. Retrocompatível (método
  novo, não quebra contrato existente).

- Correção de contraste (O-006): light `text-3` desacoplado do `--oby` e escurecido
  para `#56758A` (~4,88:1 sobre superfície branca, passa AA). `palette::light::TEXT_3`
  agora é `Hsla` próprio (não alias de `OBY`), propagado automaticamente ao tema e
  a todos os consumidores (`muted_foreground`, `tab_foreground`,
  `table_foot_foreground`, metadados, pontuação JSON).
