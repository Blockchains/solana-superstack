# Solana SuperStack

Catalogue of Solana hackathon projects (Colosseum) forked into the Blockchains account, plus a small set of attributed, CI-built modules.

- `catalog.json`: machine-readable catalogue (356 repos, refreshed nightly from Blockchains/colosseum-forker state)
- `CATALOG.md` and `repos/*.md`: one page per repo linking to the Blockchains fork
- `modules/`: permissively licensed programs copied verbatim, with LICENSE and NOTICE crediting the original repo

Categories and frameworks come from keyword heuristics over GitHub metadata, READMEs and manifests. `builds` is `unverified` for every catalogued repo; only code under `modules/` is built in CI.

## By category
- Other: 210
- Consumer: 48
- Infra/Tooling: 43
- NFTs: 39
- DeFi: 35
- Social: 35
- Gaming: 22
- Payments: 15
- DAO/Governance: 13
- DePIN: 12
- Identity/Privacy: 11
- AI agents: 7

## By framework
- none detected: 205
- Anchor: 91
- Client-only (web3.js/kit): 87
- Native (solana-program): 44
- Steel: 1

## Modules (see modules-ci workflow for status)
| Module | Origin | Category |
|---|---|---|
| gree | https://github.com/asimnecip/gree | Infra/Legal (agreements) |
| kryptpay | https://github.com/KryptPay/kryptpay-solana-contract | Payments (payments + reward token) |
| poe | https://github.com/ProofOfEstimate/poe-solana | DAO/Social (proof of estimate) |
| predix | https://github.com/ymbozkurt/Predix | DeFi (prediction market) |
| sca | https://github.com/Sergio-Flores-Torres/sca | Payments/Commerce (native) |
| valhalla | https://github.com/restacked-dev/valhalla | DeFi (LP/token locker) |

Each module's original authors keep their copyright; see that module's LICENSE and NOTICE files.
