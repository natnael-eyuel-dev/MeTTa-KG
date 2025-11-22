INSERT INTO tokens (
  id, 
  code, 
  description, 
  namespace,
  creation_timestamp,
  permission_read,
  permission_write,
  permission_share_share,
  permission_share_read,
  permission_share_write
) VALUES (
  0, 
  lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-' || hex(randomblob(2)) || '-' || hex(randomblob(2)) || '-' || hex(randomblob(6))),
  'DeepFunding KG', 
  '/',
  CURRENT_TIMESTAMP,
  1,
  1,
  1,
  1,
  1
);
