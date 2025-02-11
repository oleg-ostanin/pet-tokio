-- Add a table update notification function
CREATE OR REPLACE FUNCTION table_update_notify() RETURNS trigger AS $$
DECLARE
  order_id int;
  user_id int;
--  value varchar;
BEGIN
  IF TG_OP = 'INSERT' OR TG_OP = 'UPDATE' THEN
    order_id = NEW.order_id;
    user_id = NEW.user_id;
--    value = NEW.val;
  ELSE
    order_id = OLD.order_id;
    user_id = OLD.user_id;
--    value = OLD.val;
  END IF;
  PERFORM pg_notify('table_update', json_build_object('table', TG_TABLE_NAME, 'order_id', order_id, 'user_id', user_id, 'action_type', TG_OP)::text);
--  PERFORM pg_notify('table_update', json_build_object('table', TG_TABLE_NAME, 'order_id', order_id, 'user_id', user_id, 'value', value, 'action_type', TG_OP)::text);
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