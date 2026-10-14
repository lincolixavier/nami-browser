# Roadmap

Browser minimalista escrito do zero em Rust, seguindo a estrutura de
[Web Browser Engineering](https://browser.engineering/). Interface: apenas tabs e
bookmarks.

## Fronteira com o sistema

- Zero crates.
- TLS: Security.framework do macOS via FFI (única exceção de engine).
- Janela: FFI mínimo com Cocoa, só para abrir janela, receber eventos e blitar o
  framebuffer. Todo o resto é próprio: rasterizador, parser TrueType, glifos.

## Fases

| # | Fase | Capítulo | Status |
|---|------|----------|--------|
| 1 | Rede: URL, HTTP/1.1, TLS, redirects, chunked, `file:`, `data:`, `view-source:` | 1 | feito |
| 2 | Janela: FFI Cocoa, framebuffer, rasterizador de retângulos, scroll | 2 | |
| 3 | Texto: parser TrueType, rasterização de glifos, cache, métricas, quebra de linha | 3 | |
| 4 | Árvore HTML: tokenizer, tree builder, tags implícitas | 4 | |
| 5 | Layout: árvore de layout block/inline, display list | 5 | |
| 6 | CSS: parser, seletores, cascata, herança | 6 | |
| 7 | Chrome: tabs, bookmarks, links, histórico, `Cmd+L` para editar URL no próprio tab | 7 | |
| 8 | Formulários: input, POST | 8 | |
| 9 | JavaScript: lexer, parser, interpretador, bindings de DOM, eventos | 9 | |
| 10 | Cookies e same-origin | 10 | |
| 11 | Efeitos visuais: alpha, opacity, border-radius, blend | 11 | |
| 12 | Scheduling, threads, animações, compositing | 12–13 | |
| 13 | Imagens (inflate, PNG) e gzip no HTTP | 15 | |
| 14 | Acessibilidade e invalidação incremental | 14, 16 | |

## Interface

Sem barra de endereço, sem botões. Uma faixa de tabs e uma faixa de bookmarks.
Navegação por teclado: `Cmd+T` novo tab, `Cmd+W` fecha, `Cmd+L` edita a URL do
tab ativo, `Cmd+D` adiciona bookmark, `Cmd+[`/`Cmd+]` voltar/avançar.
