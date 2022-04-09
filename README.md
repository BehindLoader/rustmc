# rustmc
Реализация игрового Minecraft сервера на Rust

![image](https://user-images.githubusercontent.com/7967826/162589453-09cc1240-cfd8-401d-ba6a-e8dcb6fe3de7.png)

Пока реализован только пинг сервера в "Сетевой Игре" и авторизация клиента майнкрафта (без проверки лицензионной копии игрока). Сервер не реализует и 10% необходимого функционала, через него "не поиграть", но в теории возможно когда-нибудь кому-нибудь репозиторий может быть полезен как демонстрация протокола

[Документация протокола Minecraft](https://wiki.vg/Protocol)

Другие реализации Minecraft сервера:
- [pyCraft на Python](https://github.com/ammaraskar/pyCraft) - пушка
- [feather на Rust](https://github.com/feather-rs/feather) - бомба
- [ULE на Rust](https://github.com/Distemi/ULE/) - тоже не полностью рабочая версия, но я оттуда брал примеры для парсинга пакетов TCP например типа данных VarInt
