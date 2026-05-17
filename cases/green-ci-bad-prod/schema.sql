create table deploys (
    id bigint primary key,
    service text not null,
    image_digest text not null,
    deployed_at timestamptz not null,
    ci_status text not null
);

create table edge_health_checks (
    id bigint primary key,
    service text not null,
    path text not null,
    expected_status integer not null
);
