from locust import HttpUser, task, between
import random

domains_frag = "ssl-{}.domain.{}"
domain_suffix = [
    "com",
    "net",
    "org",
    "dev"
]

MAX_DOMAINS = 100


class Basic(HttpUser):
    wait_time = between(0.5, 0.5)

    def domain(self):
        # assemble domain
        suffix_key = random.randint(0, (len(domain_suffix) - 1))
        suffix = domain_suffix[suffix_key]
        site_key = random.randint(1, MAX_DOMAINS)
        domain = domains_frag.format(site_key, suffix)

        return domain
        

    @task
    def cpu(self):
        domain = self.domain()
        self.client.get("https://{}".format(domain), verify=False)
