 INSERT INTO workspaces(name,owner_id)
 VALUES  ('acme',0),
         ('foo',0),
         ('bar',0);

-- password_hash 123456
INSERT INTO users(ws_id,email,fullname,password_hash)
VALUES (1,'tchen1@acme.org','Tyr chen','$argon2id$v=19$m=19456,t=2,p=1$jR8CP5LE/eIhzvSAWH2buw$2IQS47j1gRUnyd3BjfBO+hEs2H8korHnjzPKFJNfkAc'),
       (1,'tchen2@acme.org','Boy chen','$argon2id$v=19$m=19456,t=2,p=1$jR8CP5LE/eIhzvSAWH2buw$2IQS47j1gRUnyd3BjfBO+hEs2H8korHnjzPKFJNfkAc'),
       (1,'tchen3@acme.org','Dawd chen','$argon2id$v=19$m=19456,t=2,p=1$jR8CP5LE/eIhzvSAWH2buw$2IQS47j1gRUnyd3BjfBO+hEs2H8korHnjzPKFJNfkAc'),
       (1,'tchen4@acme.org','Zdsdd chen','$argon2id$v=19$m=19456,t=2,p=1$jR8CP5LE/eIhzvSAWH2buw$2IQS47j1gRUnyd3BjfBO+hEs2H8korHnjzPKFJNfkAc'),
       (1,'tchen5@acme.org','Tqqq chen','$argon2id$v=19$m=19456,t=2,p=1$jR8CP5LE/eIhzvSAWH2buw$2IQS47j1gRUnyd3BjfBO+hEs2H8korHnjzPKFJNfkAc'),
       (1,'tchen6@acme.org','Tsfd chen','$argon2id$v=19$m=19456,t=2,p=1$jR8CP5LE/eIhzvSAWH2buw$2IQS47j1gRUnyd3BjfBO+hEs2H8korHnjzPKFJNfkAc'),
       (1,'tchen7@acme.org','Tbgb chen','$argon2id$v=19$m=19456,t=2,p=1$jR8CP5LE/eIhzvSAWH2buw$2IQS47j1gRUnyd3BjfBO+hEs2H8korHnjzPKFJNfkAc'),
       (1,'tchen8@acme.org','Tmk chen','$argon2id$v=19$m=19456,t=2,p=1$jR8CP5LE/eIhzvSAWH2buw$2IQS47j1gRUnyd3BjfBO+hEs2H8korHnjzPKFJNfkAc');

 INSERT INTO chats (ws_id, name ,type,members)
 VALUES (1,'general','public_channel','{1,2,3,4,5}'),
        (1,'private','private_channel','{1,2,3}');

INSERT INTO chats (ws_id,type, members)
VALUES  (1,'single','{1,2}'),
        (1, 'group', '{1,3,4}');