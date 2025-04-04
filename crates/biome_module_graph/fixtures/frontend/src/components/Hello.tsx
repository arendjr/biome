export type HelloProps = {
    name: string;
};

export function Hello(props: HelloProps): JSX.Element {
    return (
        <div>
            {`Hello ${props.name}`}
        </div>
    )
}
