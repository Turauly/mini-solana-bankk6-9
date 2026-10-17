# Solana-разработка: ДЗ 6-9

**Студент:** Мухаммаддин Тураұлы  
**Сеть:** Solana Devnet  
**Инструменты:** Solana Playground, Anchor, TypeScript

В этом репозитории собраны четыре выполненных задания по Solana-разработке:

| ДЗ | Требование | Статус | Папка | On-chain |
|---|---|---|---|---|
| 6 | Создать децентрализованный on-chain профиль с использованием PDA | Выполнено | `dz6-profile-pda` | [Program 4Vyg54...vu77c](https://explorer.solana.com/address/4Vyg54rpLnq1ybfXnUdyZotWDV7Po1bT2UU3pAuvu77c?cluster=devnet) |
| 7 | Создать собственный токен: выпуск, transfer, metadata и взаимодействие через client | Выполнено | `dz7-token-2022` | [Mint B58ySQ...L4q5S](https://explorer.solana.com/address/B58ySQQw74sNYRyft4XTxtypHQUJSTjdPQjuubtL4q5S?cluster=devnet) |
| 8 | Создать работающий прототип в формате мини-хакатона | Выполнено | `dz8-tip-jar-mvp` | [Program H3qZWk...uGAQ](https://explorer.solana.com/address/H3qZWkqCEYDRyuTpHYLk8sChPFAqjWsnSTq5eUG4uGAQ?cluster=devnet) |
| 9 | Изучить реальные протоколы экосистемы и интегрировать DeFi-функциональность в приложение | Выполнено | `dz9-defi-pyth` | [Program H3qZWk...uGAQ](https://explorer.solana.com/address/H3qZWkqCEYDRyuTpHYLk8sChPFAqjWsnSTq5eUG4uGAQ?cluster=devnet) |

## ДЗ 6: Accounts, PDAs и архитектура программ

Задание выполнено через Anchor-программу `profile_pda`.

Что сделано:

- создан on-chain профиль пользователя;
- адрес профиля рассчитывается как PDA по seeds `["profile", wallet]`;
- реализованы инструкции `create_profile`, `update_profile`, `delete_profile`;
- добавлены проверки длины `username` и `bio`;
- обновлять и удалять профиль может только владелец;
- клиент `client.ts` создаёт, обновляет и читает профиль из Devnet.

Основные файлы:

- `dz6-profile-pda/lib.rs` - Anchor-программа;
- `dz6-profile-pda/client.ts` - TypeScript-клиент для Solana Playground.

Program ID:

```text
4Vyg54rpLnq1ybfXnUdyZotWDV7Po1bT2UU3pAuvu77c
```

## ДЗ 7: SPL Tokens и Token Extensions

Задание выполнено через Token-2022.

Что сделано:

- создан собственный токен `Mukhammaddin Token`;
- symbol: `MUKH`;
- decimals: `6`;
- использованы расширения `MetadataPointer` и `TokenMetadata`;
- выполнен выпуск токена через `mintTo`;
- выполнен перевод через `transferChecked`;
- выполнено сжигание части токенов через `burnChecked`;
- клиент читает supply, metadata и балансы.

Основной файл:

- `dz7-token-2022/client.ts`

Mint:

```text
B58ySQQw74sNYRyft4XTxtypHQUJSTjdPQjuubtL4q5S
```

## ДЗ 8: PK1 Build Sprint - первый Web3 MVP

Задание выполнено как MVP `Tip Jar`.

Идея проекта: on-chain копилка донатов. Автор создаёт копилку с целью, любой кошелёк может отправить SOL с сообщением, а вывести средства может только владелец.

Что сделано:

- реализована инструкция `create_jar`;
- реализована инструкция `donate`;
- реализована инструкция `withdraw`;
- добавлен PDA-аккаунт копилки по seeds `["jar", owner]`;
- добавлен счётчик донатов и сумма `total_donated`;
- сохраняются последний донатер и последнее сообщение;
- добавлено событие `DonationEvent`;
- чужой `withdraw` отклоняется программой.

Основные файлы:

- `dz8-tip-jar-mvp/lib.rs` - Anchor-программа;
- `dz8-tip-jar-mvp/client.ts` - TypeScript-клиент.

Program ID:

```text
H3qZWkqCEYDRyuTpHYLk8sChPFAqjWsnSTq5eUG4uGAQ
```

## ДЗ 9: DeFi на Solana

Задание выполнено как развитие `Tip Jar` с DeFi-интеграцией через Pyth.

Что сделано:

- изучены протоколы экосистемы Solana: Jupiter, Pyth, Raydium, Orca, Meteora, Kamino, Marinade, Jito, Drift;
- в программу добавлена инструкция `donate_usd`;
- программа читает on-chain цену SOL/USD из аккаунта Pyth;
- проверяется владелец аккаунта Pyth Receiver;
- проверяется feed id SOL/USD;
- проверяется уровень верификации цены;
- проверяется свежесть цены;
- сумма в долларах пересчитывается в lamports внутри программы;
- подмена oracle account отклоняется.

Основные файлы:

- `dz9-defi-pyth/lib.rs` - обновлённая Anchor-программа Tip Jar + Pyth;
- `dz9-defi-pyth/client.ts` - TypeScript-клиент для `donate_usd`.

Pyth SOL/USD price account:

```text
7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE
```

Program ID:

```text
H3qZWkqCEYDRyuTpHYLk8sChPFAqjWsnSTq5eUG4uGAQ
```

## Структура репозитория

```text
.
├── dz6-profile-pda
│   ├── lib.rs
│   └── client.ts
├── dz7-token-2022
│   └── client.ts
├── dz8-tip-jar-mvp
│   ├── lib.rs
│   └── client.ts
├── dz9-defi-pyth
│   ├── lib.rs
│   └── client.ts
└── README.md
```

## Итог

Все четыре требования из ДЗ 6-9 выполнены:

- ДЗ 6 - on-chain профиль на PDA;
- ДЗ 7 - собственный SPL Token-2022 с metadata;
- ДЗ 8 - рабочий Web3 MVP `Tip Jar`;
- ДЗ 9 - DeFi-интеграция через Pyth oracle.
