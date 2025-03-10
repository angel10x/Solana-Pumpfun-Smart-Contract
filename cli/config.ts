import idl from "../target/idl/pumpfun_forking.json";
import { PumpfunForking } from "../target/types/pumpfun_forking";

const program = new anchor.Program(idl as PumpfunForking);

const network = "devnet"

export { program, network };
