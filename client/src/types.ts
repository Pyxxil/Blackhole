type RR = { domain: string; query_type: number; class: number; ttl: number; data_length: number };

type MX = { MX: { record: RR; priority: number } };
type TXT = { TXT: { record: RR; data: string } };
type A = { A: { record: RR; addr: string } };
type AAAA = { AAAA: { record: RR; addr: string } };
type NS = { NS: { record: RR; host: string } };
type CNAME = { CNAME: { record: RR; host: string } };

type Answer = MX | TXT | A | AAAA | NS | CNAME;

type Rule = {
    action: unknown;
    domain: string;
    ty: "Deny" | "Allow";
};

interface Request {
    client: string;
    answers: Answer[];
    question: { name: string; qtype: string };
    status: string;
    elapsed: number;
    timestamp: number;
    cached: boolean;
    rule: Rule | null;
}

type Requests = Request[];

interface Cache {
    hits: number;
    misses: number;
    size: number;
}

interface Average {
    count: number;
    average: number;
}

interface Config {
    filter: { name: string; url: string }[];
    schedule: { name: string; schedule: string }[];
    upstream: { ip: string; port: number }[];
}

export type { Answer, Request, Requests, Cache, Average, Config };
