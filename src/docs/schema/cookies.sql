DROP TABLE cookies;
CREATE TABLE cookies (
    user_nid INTEGER NOT NULL,
    token_cookie VARCHAR(600),
    app_computer_name VARCHAR(255),
    app_ip_address VARCHAR(255),
    last_update TIMESTAMPTZ NOT NULL DEFAULT with_timezone(),
    app_name VARCHAR(30),
    PRIMARY KEY (user_nid)
);
