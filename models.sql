CREATE TABLE items (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(255) NOT NULL,
    quantity INT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);

CREATE TABLE users (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    password VARCHAR(255) NOT NULL,
    balance int NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE users
    ADD COLUMN all_orders JSON,
    ADD COLUMN pending_orders JSON,
    ADD COLUMN fufilled_orders JSON,
    ADD COLUMN total_profit INT,
    ADD COLUMN total_losses INT,
    ADD COLUMN is_admin TINYINT,
    ADD COLUMN is_approved TINYINT,
    ADD COLUMN is_blocked TINYINT,
    ADD COLUMN connections JSON,
    ADD COLUMN grof_points INT,

ALTER TABLE users ADD COLUMN role VARCHAR(190) NOT NULL;

ALTER TABLE users ADD COLUMN phone_number VARCHAR(20);
ALTER TABLE users ADD COLUMN role VARCHAR(50) DEFAULT 'student';
ALTER TABLE users ADD COLUMN address VARCHAR(190);

-- Drop existing cart table
DROP TABLE cart;

-- Create new cart table
CREATE TABLE cart (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    paid TINYINT NOT NULL DEFAULT 0, -- 0 = unpaid, 1 = paid
    package VARCHAR(20) NOT NULL, -- family, student
    email VARCHAR(255) NOT NULL,
    total_order_amount BIGINT NOT NULL, -- In kobo
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);

-- Create orders table
CREATE TABLE orders (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    cart_id BIGINT NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'confirmed', -- confirmed, shipped, delivered
    email VARCHAR(255) NOT NULL,
    address VARCHAR(190) NOT NULL, -- Matches users.address
    delivery_date DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (cart_id) REFERENCES cart(id)
);
