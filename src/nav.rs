use leptos::prelude::*;

#[component]
pub fn Nav() -> impl IntoView {
    view! {
        <div class="sticky top-0 left-0 w-full bg-gray dark:bg-gray-900 p-4 border-b-2 border-gray-500 text-tulip flex flex-row justify-between">
            <a href="/">
                <img src="/Tulip.svg" class="h-10 mx-4 hidden dark:flex"/>
                <img src="/Tulip.svg" class="h-10 mx-4 dark:hidden flex"/>
            </a>
            <div class="space-x-4 flex self-center">
                <a href="/realtimekeys" class="hover:underline flex pt-2">
                    <span class="material-symbols-outlined pb-2 align-middle px-2">
                    "key"
                    </span>
                    <span class="hidden md:inline md:pr-4">"Keys (Admin)"</span>
                </a>
                <a href="/chateaux" class="hover:underline flex pt-2">
                    <span class="material-symbols-outlined pb-2 align-middle px-2">
                    "account_tree"
                    </span>
                    <span class="hidden md:inline md:pr-4">"Châteaux"</span>
                </a>
            </div>
        </div>
    }
}
