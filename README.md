# oxide-arbiter
A memory safe matching engine

oxide-arbiter is a high-performance order matching engine written in Rust that implements a Centralized Limit Order Book (CLOB). It provides the core infrastructure for building trading systems across various asset classes including stocks, cryptocurrencies, and commodities.

The engine supports both limit and market orders with comprehensive time-in-force enforcement (GTC, IOC, FOK, DAY). It implements price-time priority matching, ensuring orders are filled fairly based on price first, then timestamp. The system handles partial fills, records all executed trades, and provides efficient order management capabilities with O(1) lookups.

Built in Rust for memory safety and performance, oxide-arbiter offers a production-ready foundation for developers looking to build exchange platforms or integrate order matching functionality into their trading systems.

<img width="1024" height="1024" alt="Gemini_Generated_Image_dwxbg7dwxbg7dwxb" src="https://github.com/user-attachments/assets/99cae915-fe0a-41fe-bca5-093d04dbb277" />
