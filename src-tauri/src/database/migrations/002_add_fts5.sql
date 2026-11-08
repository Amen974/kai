CREATE VIRTUAL TABLE history_fts5 USING fts5(
    item_type UNINDEXED,
    item_id UNINDEXED,
    history_id UNINDEXED,
    content
);

CREATE TRIGGER history_ai
AFTER INSERT ON historys
BEGIN
    INSERT INTO history_fts5 (
        item_type,
        item_id,
        history_id,
        content
    )
    VALUES (
        'history',
        NEW.id,
        NEW.id,
        NEW.title
    );
END;

CREATE TRIGGER history_ad
AFTER DELETE ON historys
BEGIN
    DELETE FROM history_fts5
    WHERE history_id = OLD.id;
END;

CREATE TRIGGER history_au
AFTER UPDATE OF title ON historys
BEGIN
    UPDATE history_fts5
    SET content = NEW.title
    WHERE item_type = 'history'
      AND item_id = OLD.id;
END;

CREATE TRIGGER message_ai
AFTER INSERT ON messages
WHEN NEW.role = 'user'
BEGIN
    INSERT INTO history_fts5 (
        item_type,
        item_id,
        history_id,
        content
    )
    VALUES (
        'message',
        NEW.id,
        NEW.history_id,
        NEW.content
    );
END;

CREATE TRIGGER message_ad
AFTER DELETE ON messages
WHEN OLD.role = 'user'
BEGIN
    DELETE FROM history_fts5
    WHERE item_type = 'message'
      AND item_id = OLD.id;
END;

CREATE TRIGGER message_au
AFTER UPDATE OF content ON messages
WHEN NEW.role = 'user'
BEGIN
    UPDATE history_fts5
    SET content = NEW.content
    WHERE item_type = 'message'
      AND item_id = OLD.id;
END;

INSERT INTO history_fts5 (item_type, item_id, history_id, content)
SELECT 'history', id, id, title FROM historys;

INSERT INTO history_fts5 (item_type, item_id, history_id, content)
SELECT 'message', id, history_id, content FROM messages WHERE role = 'user';
