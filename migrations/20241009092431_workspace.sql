-- Add migration script here
-- add super user 0

-- workspace   for  user
CREATE TABLE IF NOT EXISTS workspaces (
    id bigserial PRIMARY KEY ,
    name varchar(32) NOT NULL UNIQUE ,
    owner_id bigint NOT NULL REFERENCES users(id),

    created_at timestamptz DEFAULT CURRENT_TIMESTAMP
);

-- alter users table to add ws_id
ALTER  TABLE users
    ADD COLUMN ws_id bigint REFERENCES workspaces(id);

-- alter users table to add ws_id
ALTER  TABLE chats
    ADD COLUMN ws_id bigint REFERENCES workspaces(id);

BEGIN ;
INSERT  INTO  users(id, fullname, email, password_hash )
VALUES (0,'super user','super@163.com', '');
INSERT INTO  workspaces(id,name,owner_id)
VALUES (0,'none',0);
UPDATE
    users
        SET
            ws_id = 0
WHERE
    id = 0;
COMMIT ;

-- alter user table to make ws_id not null
ALTER TABLE users
    ALTER COLUMN ws_id SET NOT NULL;
