DROP TABLE users;
CREATE TABLE users (
    user_id SERIAL PRIMARY KEY,
    password VARCHAR(100) NOT NULL,
    email VARCHAR(100) NOT NULL,
    handphone VARCHAR(100) NOT NULL,
    register_date TIMESTAMPTZ NOT NULL DEFAULT with_timezone(),
    web_cif_id INTEGER NOT NULL,
    disable_login BOOLEAN NOT NULL,
    activate_code VARCHAR(100),
    activate_time TIMESTAMPTZ,
    last_login TIMESTAMPTZ,
    client_category INTEGER,
    last_resend_otp TIMESTAMPTZ,
    otp_generated_link VARCHAR(100),
    reset_password_key VARCHAR(100),
    reset_password_flag BOOLEAN,
    reset_password_date TIMESTAMPTZ,
    otp_generated_link_date TIMESTAMPTZ,
    count_resend_activation INTEGER NOT NULL DEFAULT 0,
    picture TEXT,
    google_id TEXT
);
