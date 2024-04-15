## Rust Auth Service
This application serves as a template for a micro-service auth app. It implements Axum as its primary web framework and MongoDB for storing data.

### Features:
- Oauth2 Access and Refresh tokens
- MongoDB & Axum
- User registration
- Manual login
- Google login
- Logout

### Patterns:
1. User builder pattern
2. Typestate pattern for email and password fields
3. Http response builder pattern

### Directories:
- /handlers: This directory contains all the main entry point functions of the API endpoints.

- /services: Here lies the application logic. All services are functions that serve as a bridge between the database and the API.

- /database: This directory declares functions that directly communicates with MongoDB.

- /models: The user model is declared here, containing the entire structure of the user, as well as its validations.

- /utils: This directory stores reusable chunks of logic.
