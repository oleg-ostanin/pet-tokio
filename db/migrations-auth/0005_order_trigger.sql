-- Add a table update notification function
CREATE OR REPLACE FUNCTION table_update_notify() RETURNS trigger AS $$
DECLARE
  order_id bigint;
  user_id bigint;
  content Json;
  status order_status;
  created_at timestamp with time zone;
  updated_at timestamp with time zone;
--  value varchar;
BEGIN
  IF TG_OP = 'INSERT' OR TG_OP = 'UPDATE' THEN
    order_id = NEW.order_id;
    user_id = NEW.user_id;
    content = NEW.content;
    status = NEW.status;
    created_at = NEW.created_at;
    updated_at = NEW.updated_at;

  ELSE
    order_id = OLD.order_id;
    user_id = OLD.user_id;
    content = OLD.content;
    status = OLD.status;
    created_at = OLD.created_at;
    updated_at = OLD.updated_at;

  END IF;
  PERFORM pg_notify('table_update', json_build_object(
  'table', TG_TABLE_NAME,
  'order_id', order_id,
  'user_id', user_id,
  'content', content,
  'status', status,
  'created_at', created_at,
  'updated_at', updated_at,
  'action_type', TG_OP
  )::text);
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Add UPDATE row trigger
DROP TRIGGER IF EXISTS order_notify_update ON order_info;
CREATE TRIGGER order_notify_update AFTER UPDATE ON order_info FOR EACH ROW EXECUTE PROCEDURE table_update_notify();

-- Add INSERT row trigger
DROP TRIGGER IF EXISTS order_notify_insert ON order_info;
CREATE TRIGGER order_notify_insert AFTER INSERT ON order_info FOR EACH ROW EXECUTE PROCEDURE table_update_notify();

-- Add DELETE row trigger
DROP TRIGGER IF EXISTS order_notify_delete ON order_info;
CREATE TRIGGER order_notify_delete AFTER DELETE ON order_info FOR EACH ROW EXECUTE PROCEDURE table_update_notify();