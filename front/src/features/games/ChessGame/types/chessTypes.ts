export type PieceType = "pawn" | "rook" | "knight" | "bishop" | "queen" | "king" | null | undefined;
export type ModifiersType = "sniper" | "canon" | "battlefield";
export type PieceColor = "white" | "black";

export type Piece = { id: string; type: PieceType; piece_modifier?: any; square_modifier?: any; color: PieceColor; moveset?: string[] };