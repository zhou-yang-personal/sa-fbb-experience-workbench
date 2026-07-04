type Props = {
  title: string;
  detail: string;
  tone?: 'normal' | 'warning' | 'blocked' | 'success';
};

export function NextActionHint({ title, detail, tone = 'normal' }: Props) {
  return (
    <section className={`next-action-hint next-action-${tone}`}>
      <span>Next</span>
      <strong>{title}</strong>
      <small>{detail}</small>
    </section>
  );
}
