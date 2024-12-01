import type { Route } from './+types/refinements';

export async function clientLoader() {
  // you can now fetch data here
  return {
    title: 'Refinements page',
    age: 20
  };
}

export default function Component({
  loaderData
}: {
  loaderData?: Route.ComponentProps['loaderData'];
}) {
  return (
    <div>
      <div className="flex flex-row justify-between items-center">
        <h1 className="text-4xl font-bold text-gray-800 dark:text-white">
          {loaderData?.title ?? 'Refinements'}
        </h1>
      </div>
    </div>
  );
}
