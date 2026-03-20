-- Create product foundations table
CREATE TABLE IF NOT EXISTS product_foundations (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- Create junction table for products and foundations
CREATE TABLE IF NOT EXISTS product_foundation_relations (
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    foundation_id UUID NOT NULL REFERENCES product_foundations(id) ON DELETE CASCADE,
    PRIMARY KEY (product_id, foundation_id)
);
