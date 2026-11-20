# Proveniência de recursos

| Arquivo | SHA-256 | Autor / origem | Ferramenta | Licença / direitos | Data | Estado |
|---|---|---|---|---|---|---|
| `conceito/logo-chatgpt.png` | `872f5753b6b69e76398ee957b03b1416c97646d340ec8184654f8ab14ee84d60` | Gerado pelo responsável do projeto | ChatGPT (geração de imagem da OpenAI) | Pelos termos de uso da OpenAI, o usuário detém os direitos sobre o resultado (termos a conferir na versão vigente). A proteção autoral de imagem gerada por IA é incerta e depende da jurisdição | 2026-11-17 | Conceito de referência; não é usado como logo oficial |
| `logo.svg` (mestre) | `6ece8183d96651a4ac383e59c7bd9b8f1839d5fff9bdb3cf1b07c3fd36c350b8` | Desenhado em SVG à mão pelo assistente de IA (Claude) a pedido do responsável, simplificando o conceito acima (cuia + bomba + erva + monograma "C" + bolhas) | Editor de texto (SVG escrito à mão) | Do projeto, sob a licença do repositório; sem registro de marca (DT-25) | 2026-11-17 | **Logo oficial** |
| `logo-icone.svg` | `59f8a27707e631ad203902be8b4b6755258a05b05f2e55a50194bc8aa8d962e4` | Variante do mestre (sem bolhas, bomba mais grossa para 16–48 px) | Idem | Idem | 2026-11-17 | Ícone oficial |
| `logo.png`, `png/logo-{1024,512,256}.png` | — | Renderizações de `logo.svg` | rsvg-convert | Idem | 2026-11-17 | Derivados (regenerar a partir do SVG) |
| `png/icone-{128,64,48,32,16}.png`, `icone.ico` | — | Renderizações de `logo-icone.svg` | rsvg-convert, ImageMagick | Idem | 2026-11-17 | Derivados |

Para regenerar os derivados: `rsvg-convert -w N -h N assets/logo.svg -o assets/png/logo-N.png` (idem para o ícone) e `magick png/icone-{16,32,48,64}.png icone.ico`.
