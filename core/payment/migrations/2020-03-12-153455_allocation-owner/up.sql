-- Your SQL goes here
DROP TABLE pay_allocation;

CREATE TABLE pay_allocation(
    owner_id        VARCHAR(50) NOT NULL,
    id              VARCHAR(50) NOT NULL PRIMARY KEY,
    total_amount    VARCHAR(32) NOT NULL,
    consumed_amount VARCHAR(32) NOT NULL,
    timeout         DATETIME NULL,
    make_deposit    BOOLEAN NOT NULL DEFAULT false
);

DROP TABLE pay_payment;

CREATE TABLE pay_payment(
    owner_id        VARCHAR(50) NOT NULL,
    id              VARCHAR(50) NOT NULL PRIMARY KEY,
    payer_id        VARCHAR(50) NOT NULL,
    payee_id        VARCHAR(50) NOT NULL,
    amount          VARCHAR(32) NOT NULL, --NUMBER(77,18) NOT NULL,
    ts              DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    allocation_id   VARCHAR(50) NULL,
    details         BLOB NOT NULL,
    CONSTRAINT pay_payment_fk1
        FOREIGN KEY(allocation_id)
        REFERENCES pay_allocation (id)
        ON DELETE SET NULL
);


