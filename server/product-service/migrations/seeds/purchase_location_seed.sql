BEGIN;

INSERT INTO purchase_location (name) VALUES
    ('SM, Dragana SC'),
    ('SM, Turi'),
    ('BT, Market'),
    ('BT, Turi'),
    ('BT, Glavna SC'),
    ('Vrbas, Issitex SC'),
    ('Vrbas, Topline SC'),
    ('Sombor, Market'),
    ('Sombor, Tex SC'),
    ('Subotica, Market'),
    ('Subotica, SC'),
    ('Kanjiza, Issitex SC'),
    ('Senta, SC'),
    ('Becej, SC'),
    ('Limundo'),
    ('NS, SC'),
    ('NS, Najlon'),
    ('Kikinda, SC'),
    ('KP')
ON CONFLICT DO NOTHING;

COMMIT;
